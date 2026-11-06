use crate::{
    cluster::segment,
    layout::{Layer, PreparedLayout},
    metrics::{self, Confusion, KeyStats, SessionSummary},
    normalize::{accepted_orders, canonicalize},
    plan::{plan, PlanError, PlannedStroke},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    StopOnError,
    Flow,
}
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent<'a> {
    pub code: &'a str,
    pub shift: bool,
    pub altgr: bool,
    pub ctrl: bool,
    pub meta: bool,
    pub repeat: bool,
    pub timestamp_ms: u64,
}
impl<'a> KeyEvent<'a> {
    pub fn new(code: &'a str, timestamp_ms: u64) -> Self {
        Self {
            code,
            shift: false,
            altgr: false,
            ctrl: false,
            meta: false,
            repeat: false,
            timestamp_ms,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyResult {
    pub ignored: bool,
    pub correct: bool,
    pub produced: String,
    pub expected_next: Option<PlannedStroke>,
    pub cluster_done: bool,
    pub finished: bool,
    pub cluster_index: usize,
    pub typed: String,
    pub error_position: Option<usize>,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct Progress {
    cluster: u32,
    prefix: u16,
    variant: u8,
}
/// Sixteen bytes per event, with source key/layer and undo position, not Strings.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct StrokeRecord {
    elapsed: u32,
    cluster: u32,
    key: u16,
    prefix: u16,
    layer: u8,
    variant: u8,
    flags: u8,
    _pad: u8,
}
const CORRECT: u8 = 1;
const ERROR: u8 = 2;
const BACKSPACE: u8 = 4;
const DUPLICATE: u8 = 8;
const REMOVED: u8 = 16;
const ENTER: u16 = u16::MAX;
const UNKNOWN: u16 = u16::MAX - 1;

#[derive(Debug, Clone)]
struct TargetCluster {
    orders: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCheckpoint {
    target: String,
    policy: Policy,
    strict: bool,
    progress: Progress,
    summary: SessionSummary,
    typed: String,
}
pub struct Session {
    pub layout: PreparedLayout,
    pub policy: Policy,
    pub strict: bool,
    target: String,
    clusters: Vec<TargetCluster>,
    progress: Progress,
    records: Vec<StrokeRecord>,
    undo: Vec<usize>,
    started: Option<u64>,
    last_time: u64,
    pause_threshold: u64,
    error_at: Option<(u32, u16)>,
    previous: SessionSummary,
    previous_text: String,
}
impl Session {
    pub fn new(
        target: &str,
        layout: PreparedLayout,
        policy: Policy,
        strict: bool,
    ) -> Result<Self, PlanError> {
        if target.chars().count() > 20_000 {
            return Err(PlanError {
                code: "target_too_long".into(),
                offset: 0,
                character: String::new(),
            });
        }
        plan(target, &layout)?;
        let target = if strict {
            target.to_string()
        } else {
            canonicalize(target)
        };
        let clusters = segment(&target)
            .iter()
            .map(|c| TargetCluster {
                orders: accepted_orders(&target[c.start..c.end], strict),
            })
            .collect();
        Ok(Self {
            layout,
            policy,
            strict,
            target,
            clusters,
            progress: Progress::default(),
            records: Vec::new(),
            undo: Vec::new(),
            started: None,
            last_time: 0,
            pause_threshold: 10_000,
            error_at: None,
            previous: SessionSummary::default(),
            previous_text: String::new(),
        })
    }
    pub fn target(&self) -> &str {
        &self.target
    }
    pub fn restart(&mut self) {
        self.progress = Progress::default();
        self.records.clear();
        self.undo.clear();
        self.started = None;
        self.last_time = 0;
        self.error_at = None;
        self.previous = SessionSummary::default();
        self.previous_text.clear();
    }
    pub fn set_pause_threshold(&mut self, ms: u64) {
        self.pause_threshold = ms;
    }
    pub fn finished(&self) -> bool {
        self.progress.cluster as usize >= self.clusters.len()
    }
    pub fn cluster_index(&self) -> usize {
        self.progress.cluster as usize
    }
    fn prefix(&self, p: Progress) -> &str {
        if let Some(cluster) = self.clusters.get(p.cluster as usize) {
            &cluster.orders[p.variant as usize][..p.prefix as usize]
        } else {
            ""
        }
    }
    pub fn expected_next(&self) -> Option<PlannedStroke> {
        self.expected_at(self.progress)
    }
    fn expected_at(&self, progress: Progress) -> Option<PlannedStroke> {
        let cluster_index = progress.cluster as usize;
        let cluster = self.clusters.get(cluster_index)?;
        let rest = &cluster.orders[progress.variant as usize][progress.prefix as usize..];
        if rest.starts_with('\n') {
            return Some(PlannedStroke {
                code: "Enter".into(),
                layer: Layer::Base,
                produces: "\n".into(),
                cluster_index,
                finger: "R5".into(),
            });
        }
        // Include later clusters for multi-code-point keys spanning singletons.
        let joined;
        let max_output_bytes = self
            .layout
            .outputs
            .first()
            .map(|o| o.produces.len())
            .unwrap_or(0);
        let rest = if rest.len() < max_output_bytes {
            joined = std::iter::once(rest)
                .chain(
                    self.clusters
                        .iter()
                        .skip(cluster_index + 1)
                        .take(max_output_bytes - rest.len())
                        .map(|c| c.orders[0].as_str()),
                )
                .collect::<String>();
            joined.as_str()
        } else {
            rest
        };
        self.layout.longest(rest).map(|o| PlannedStroke {
            code: o.code.clone(),
            layer: o.layer,
            produces: o.produces.clone(),
            cluster_index,
            finger: o.finger.clone(),
        })
    }
    fn apply(&self, produced: &str) -> Option<Progress> {
        let mut p = self.progress;
        for c in produced.chars() {
            let cluster = self.clusters.get(p.cluster as usize)?;
            let mut candidate = self.prefix(p).to_string();
            candidate.push(c);
            let variant = cluster
                .orders
                .iter()
                .position(|order| order.starts_with(&candidate))?;
            if candidate == cluster.orders[variant] {
                p = Progress {
                    cluster: p.cluster + 1,
                    ..Progress::default()
                };
            } else {
                p.prefix = candidate.len() as u16;
                p.variant = variant as u8;
            }
        }
        Some(p)
    }
    fn record_output(&self, r: &StrokeRecord) -> &str {
        if r.key == ENTER {
            "\n"
        } else if r.key == UNKNOWN {
            ""
        } else {
            self.layout
                .output(
                    &self.layout.codes[r.key as usize],
                    Layer::ALL[r.layer as usize],
                )
                .unwrap_or("")
        }
    }
    pub fn typed(&self) -> String {
        let mut text = self.previous_text.clone();
        text.extend(
            self.undo
                .iter()
                .map(|&i| self.record_output(&self.records[i])),
        );
        text
    }
    pub fn checkpoint(&self) -> SessionCheckpoint {
        SessionCheckpoint {
            target: self.target.clone(),
            policy: self.policy,
            strict: self.strict,
            progress: self.progress,
            summary: self.summary(),
            typed: self.typed(),
        }
    }
    pub fn restore(snapshot: SessionCheckpoint, layout: PreparedLayout) -> Result<Self, PlanError> {
        let mut session = Self::new(&snapshot.target, layout, snapshot.policy, snapshot.strict)?;
        let p = snapshot.progress;
        let valid = if p.cluster as usize == session.clusters.len() {
            p.prefix == 0 && p.variant == 0
        } else if let Some(c) = session.clusters.get(p.cluster as usize) {
            c.orders.get(p.variant as usize).is_some_and(|text| {
                text.is_char_boundary(p.prefix as usize) && usize::from(p.prefix) < text.len()
            })
        } else {
            false
        };
        if !valid
            || snapshot.typed.chars().count() > 40_000
            || snapshot.summary.correct > snapshot.summary.keystrokes
            || snapshot.summary.active_ms > snapshot.summary.duration_ms
        {
            return Err(PlanError {
                code: "invalid_checkpoint".into(),
                offset: 0,
                character: String::new(),
            });
        }
        session.progress = p;
        session.previous = snapshot.summary;
        session.previous_text = snapshot.typed;
        Ok(session)
    }
    fn result(
        &self,
        ignored: bool,
        correct: bool,
        produced: String,
        cluster_done: bool,
        error_position: Option<usize>,
    ) -> KeyResult {
        // The UI gets only a small change; full text is requested separately.
        KeyResult {
            ignored,
            correct,
            produced,
            expected_next: self.expected_next(),
            cluster_done,
            finished: self.finished(),
            cluster_index: self.cluster_index(),
            typed: String::new(),
            error_position,
        }
    }
    pub fn on_key(&mut self, event: KeyEvent<'_>) -> KeyResult {
        if event.repeat
            || event.meta
            || (event.ctrl && !event.altgr)
            || [
                "ShiftLeft",
                "ShiftRight",
                "ControlLeft",
                "ControlRight",
                "AltLeft",
                "AltRight",
                "MetaLeft",
                "MetaRight",
                "CapsLock",
                "Tab",
                "Escape",
            ]
            .contains(&event.code)
            || (self.started.is_some() && event.timestamp_ms < self.last_time)
        {
            return self.result(true, false, String::new(), false, None);
        }
        if self.finished() && event.code != "Backspace" {
            return self.result(true, false, String::new(), false, None);
        }
        let layer = Layer::from_modifiers(event.shift, event.altgr);
        let produced = if event.code == "Backspace" {
            String::new()
        } else if let Some(s) = self.layout.output(event.code, layer) {
            s.to_string()
        } else {
            String::new()
        };
        // Ignore non-printing unmapped function/navigation keys; an unmapped
        // physical character key is a genuine incorrect attempt.
        if produced.is_empty()
            && event.code != "Backspace"
            && !self.layout.key_indices.contains_key(event.code)
        {
            return self.result(true, false, String::new(), false, None);
        }
        let start = *self.started.get_or_insert(event.timestamp_ms);
        self.last_time = event.timestamp_ms;
        let elapsed = event
            .timestamp_ms
            .saturating_sub(start)
            .min(u32::MAX as u64) as u32;
        let key = if event.code == "Enter" {
            ENTER
        } else {
            *self.layout.key_indices.get(event.code).unwrap_or(&UNKNOWN)
        };
        let before = self.progress;
        let mut record = StrokeRecord {
            elapsed,
            cluster: before.cluster,
            key,
            prefix: before.prefix,
            layer: layer as u8,
            variant: before.variant,
            flags: 0,
            _pad: 0,
        };
        if event.code == "Backspace" {
            record.flags = BACKSPACE;
            if let Some(index) = self.undo.pop() {
                let old = &mut self.records[index];
                old.flags |= REMOVED;
                self.progress = Progress {
                    cluster: old.cluster,
                    prefix: old.prefix,
                    variant: old.variant,
                };
                self.error_at = None;
            }
            self.records.push(record);
            return self.result(false, true, String::new(), false, None);
        }
        let expected = self.expected_next();
        let applied = self.apply(&produced);
        let correct = !produced.is_empty() && applied.is_some();
        let position = (before.cluster, before.prefix);
        if correct {
            record.flags = CORRECT;
            self.progress = applied.unwrap();
            self.error_at = None;
        } else {
            record.flags = ERROR;
            if self.policy == Policy::StopOnError && self.error_at == Some(position) {
                record.flags |= DUPLICATE;
            }
            self.error_at = Some(position);
            if self.policy == Policy::Flow {
                if let Some(expected) = expected {
                    if let Some(p) = self.apply(&expected.produces) {
                        self.progress = p;
                    }
                }
            }
        }
        let index = self.records.len();
        self.records.push(record);
        if correct || self.policy == Policy::Flow {
            self.undo.push(index);
        }
        self.result(
            false,
            correct,
            produced,
            self.progress.cluster > before.cluster,
            if correct {
                None
            } else {
                Some(before.cluster as usize)
            },
        )
    }
    pub fn summary(&self) -> SessionSummary {
        let mut summary = SessionSummary::default();
        let mut keys: BTreeMap<(u16, u8), (u32, u32, u64)> = BTreeMap::new();
        let mut confusions: BTreeMap<(String, String), u32> = BTreeMap::new();
        let mut previous = 0;
        let mut previous_correct = None;
        for r in &self.records {
            let time = u64::from(r.elapsed);
            let gap = time.saturating_sub(previous);
            if gap <= self.pause_threshold {
                summary.active_ms += gap;
            }
            previous = time;
            if r.flags & BACKSPACE != 0 {
                summary.backspaces += 1;
                continue;
            }
            summary.keystrokes += 1;
            let entry = keys.entry((r.key, r.layer)).or_default();
            if r.flags & CORRECT != 0 {
                summary.correct += 1;
                entry.0 += 1;
                let hesitation = previous_correct
                    .map(|last| time.saturating_sub(last))
                    .unwrap_or(0);
                entry.2 += hesitation;
                previous_correct = Some(time);
            } else if r.flags & DUPLICATE == 0 {
                summary.errors += 1;
                entry.1 += 1;
                let expected = self
                    .expected_at(Progress {
                        cluster: r.cluster,
                        prefix: r.prefix,
                        variant: r.variant,
                    })
                    .map(|o| o.produces)
                    .unwrap_or_default();
                *confusions
                    .entry((expected, self.record_output(r).to_string()))
                    .or_default() += 1;
            }
        }
        summary.duration_ms = self
            .records
            .last()
            .map(|r| u64::from(r.elapsed))
            .unwrap_or(0);
        summary.clusters_done = self.progress.cluster;
        summary.duration_ms += self.previous.duration_ms;
        summary.active_ms += self.previous.active_ms;
        summary.keystrokes += self.previous.keystrokes;
        summary.correct += self.previous.correct;
        summary.errors += self.previous.errors;
        summary.backspaces += self.previous.backspaces;
        summary.kpm = metrics::rate(summary.correct, summary.active_ms);
        summary.wpm_equivalent = summary.kpm / 5.0;
        summary.accuracy = metrics::accuracy(summary.correct, summary.keystrokes);
        summary.stars = metrics::stars(summary.accuracy);
        summary.clusters_per_minute = metrics::rate(summary.clusters_done, summary.active_ms);
        summary.per_key = keys
            .into_iter()
            .map(|((key, layer), (hits, errors, hes))| KeyStats {
                code: if key == ENTER {
                    "Enter".into()
                } else if key == UNKNOWN {
                    "Unmapped".into()
                } else {
                    self.layout.codes[key as usize].clone()
                },
                layer: Layer::ALL[layer as usize],
                hits,
                errors,
                avg_hes_ms: if hits == 0 {
                    0.0
                } else {
                    hes as f64 / hits as f64
                },
            })
            .collect();
        summary.confusions = confusions
            .into_iter()
            .map(|((expected, typed), count)| Confusion {
                expected,
                typed,
                count,
            })
            .collect();
        for old in &self.previous.per_key {
            if let Some(current) = summary
                .per_key
                .iter_mut()
                .find(|k| k.code == old.code && k.layer == old.layer)
            {
                let hits = current.hits + old.hits;
                current.avg_hes_ms = if hits == 0 {
                    0.0
                } else {
                    (current.avg_hes_ms * current.hits as f64 + old.avg_hes_ms * old.hits as f64)
                        / hits as f64
                };
                current.hits = hits;
                current.errors += old.errors;
            } else {
                summary.per_key.push(old.clone());
            }
        }
        for old in &self.previous.confusions {
            if let Some(current) = summary
                .confusions
                .iter_mut()
                .find(|c| c.expected == old.expected && c.typed == old.typed)
            {
                current.count += old.count;
            } else {
                summary.confusions.push(old.clone());
            }
        }
        summary
    }
}
