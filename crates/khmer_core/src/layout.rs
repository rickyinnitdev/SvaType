use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    #[default]
    Base,
    Shift,
    Altgr,
    ShiftAltgr,
}
impl Layer {
    pub const ALL: [Self; 4] = [Self::Base, Self::Shift, Self::Altgr, Self::ShiftAltgr];
    pub fn from_modifiers(shift: bool, altgr: bool) -> Self {
        match (shift, altgr) {
            (false, false) => Self::Base,
            (true, false) => Self::Shift,
            (false, true) => Self::Altgr,
            (true, true) => Self::ShiftAltgr,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    pub base: Option<String>,
    pub shift: Option<String>,
    pub altgr: Option<String>,
    pub shift_altgr: Option<String>,
    pub finger: String,
    pub row: u8,
}
impl Key {
    pub fn output(&self, layer: Layer) -> Option<&str> {
        match layer {
            Layer::Base => &self.base,
            Layer::Shift => &self.shift,
            Layer::Altgr => &self.altgr,
            Layer::ShiftAltgr => &self.shift_altgr,
        }
        .as_deref()
        .filter(|s| !s.is_empty())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub id: String,
    pub name: BTreeMap<String, String>,
    pub verified: bool,
    pub keys: BTreeMap<String, Key>,
}
#[derive(Debug, Clone)]
pub struct Output {
    pub code: String,
    pub layer: Layer,
    pub produces: String,
    pub finger: String,
    pub key_index: u16,
}
#[derive(Debug, Clone)]
pub struct PreparedLayout {
    pub layout: Layout,
    pub outputs: Vec<Output>,
    pub by_first: HashMap<char, Vec<usize>>,
    pub key_indices: HashMap<String, u16>,
    pub codes: Vec<String>,
}
impl Layout {
    pub fn prepare(self) -> Result<PreparedLayout, String> {
        if self.id.is_empty() || self.keys.is_empty() || self.keys.len() > u16::MAX as usize {
            return Err("invalid_layout".into());
        }
        let mut outputs = Vec::new();
        let mut key_indices = HashMap::new();
        let mut codes = Vec::new();
        for (i, (code, key)) in self.keys.iter().enumerate() {
            if key.row > 4
                || !["L1", "L2", "L3", "L4", "L5", "R1", "R2", "R3", "R4", "R5"]
                    .contains(&key.finger.as_str())
            {
                return Err(format!("invalid_key:{code}"));
            }
            key_indices.insert(code.clone(), i as u16);
            codes.push(code.clone());
            for layer in Layer::ALL {
                if let Some(produces) = key.output(layer) {
                    outputs.push(Output {
                        code: code.clone(),
                        layer,
                        produces: produces.to_string(),
                        finger: key.finger.clone(),
                        key_index: i as u16,
                    });
                }
            }
        }
        // Stable ties prefer base, then shift, then AltGr; then physical code.
        outputs.sort_by(|a, b| {
            b.produces
                .len()
                .cmp(&a.produces.len())
                .then(a.layer.cmp(&b.layer))
                .then(a.code.cmp(&b.code))
        });
        let mut by_first: HashMap<char, Vec<usize>> = HashMap::new();
        for (i, output) in outputs.iter().enumerate() {
            by_first
                .entry(output.produces.chars().next().unwrap())
                .or_default()
                .push(i);
        }
        Ok(PreparedLayout {
            layout: self,
            outputs,
            by_first,
            key_indices,
            codes,
        })
    }
}
impl PreparedLayout {
    pub fn output(&self, code: &str, layer: Layer) -> Option<&str> {
        if code == "Enter" {
            Some("\n")
        } else {
            self.layout.keys.get(code)?.output(layer)
        }
    }
    pub fn longest(&self, text: &str) -> Option<&Output> {
        self.by_first
            .get(&text.chars().next()?)?
            .iter()
            .map(|&i| &self.outputs[i])
            .find(|o| text.starts_with(&o.produces))
    }
    pub fn reverse(&self, text: &str) -> Vec<&Output> {
        self.outputs.iter().filter(|o| o.produces == text).collect()
    }
}
