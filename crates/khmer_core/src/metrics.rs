use crate::layout::Layer;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyStats {
    pub code: String,
    pub layer: Layer,
    pub hits: u32,
    pub errors: u32,
    pub avg_hes_ms: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confusion {
    pub expected: String,
    pub typed: String,
    pub count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionSummary {
    pub duration_ms: u64,
    pub active_ms: u64,
    pub keystrokes: u32,
    pub correct: u32,
    pub errors: u32,
    pub backspaces: u32,
    pub kpm: f64,
    pub wpm_equivalent: f64,
    pub accuracy: f64,
    pub per_key: Vec<KeyStats>,
    pub confusions: Vec<Confusion>,
    pub clusters_done: u32,
    pub clusters_per_minute: f64,
    pub stars: u8,
}
pub fn stars(accuracy: f64) -> u8 {
    if accuracy >= 95.0 {
        3
    } else if accuracy >= 85.0 {
        2
    } else if accuracy >= 70.0 {
        1
    } else {
        0
    }
}
pub fn rate(count: u32, active_ms: u64) -> f64 {
    if active_ms == 0 {
        0.0
    } else {
        count as f64 * 60_000.0 / active_ms as f64
    }
}
pub fn accuracy(correct: u32, keystrokes: u32) -> f64 {
    if keystrokes == 0 {
        100.0
    } else {
        correct as f64 * 100.0 / keystrokes as f64
    }
}
pub fn aggregate(summaries: &[SessionSummary]) -> SessionSummary {
    let mut total = SessionSummary::default();
    for summary in summaries {
        total.duration_ms = total.duration_ms.saturating_add(summary.duration_ms);
        total.active_ms = total.active_ms.saturating_add(summary.active_ms);
        total.keystrokes = total.keystrokes.saturating_add(summary.keystrokes);
        total.correct = total.correct.saturating_add(summary.correct);
        total.errors = total.errors.saturating_add(summary.errors);
        total.backspaces = total.backspaces.saturating_add(summary.backspaces);
        total.clusters_done = total.clusters_done.saturating_add(summary.clusters_done);
        for key in &summary.per_key {
            if let Some(current) = total
                .per_key
                .iter_mut()
                .find(|k| k.code == key.code && k.layer == key.layer)
            {
                let hits = current.hits.saturating_add(key.hits);
                current.avg_hes_ms = if hits == 0 {
                    0.0
                } else {
                    (current.avg_hes_ms * current.hits as f64 + key.avg_hes_ms * key.hits as f64)
                        / hits as f64
                };
                current.hits = hits;
                current.errors = current.errors.saturating_add(key.errors);
            } else {
                total.per_key.push(key.clone());
            }
        }
        for confusion in &summary.confusions {
            if let Some(current) = total
                .confusions
                .iter_mut()
                .find(|c| c.expected == confusion.expected && c.typed == confusion.typed)
            {
                current.count = current.count.saturating_add(confusion.count);
            } else {
                total.confusions.push(confusion.clone());
            }
        }
    }
    total.kpm = rate(total.correct, total.active_ms);
    total.wpm_equivalent = total.kpm / 5.0;
    total.accuracy = accuracy(total.correct, total.keystrokes);
    total.stars = stars(total.accuracy);
    total.clusters_per_minute = rate(total.clusters_done, total.active_ms);
    total
}
