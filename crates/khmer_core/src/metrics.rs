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
