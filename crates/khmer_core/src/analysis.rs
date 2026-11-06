use crate::metrics::KeyStats;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakKey {
    pub code: String,
    pub score: f64,
}
pub fn weak_keys(stats: &[KeyStats], limit: usize) -> Vec<WeakKey> {
    let mut grouped = std::collections::BTreeMap::<String, (u64, u64)>::new();
    for key in stats {
        let entry = grouped.entry(key.code.clone()).or_default();
        entry.0 += key.hits as u64;
        entry.1 += key.errors as u64;
    }
    let mut keys: Vec<_> = grouped
        .into_iter()
        .filter(|(_, (_, errors))| *errors > 0)
        .map(|(code, (hits, errors))| {
            let attempts = hits + errors;
            WeakKey {
                code,
                score: errors as f64 / attempts as f64 * (attempts as f64).ln_1p(),
            }
        })
        .collect();
    keys.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.code.cmp(&b.code)));
    keys.truncate(limit);
    keys
}
/// Seeded weighted sampling; empty banks are safe. No runtime randomness or I/O.
pub fn drill_generate(
    bank: &[(String, Vec<String>)],
    weak: &[String],
    seed: u64,
    count: usize,
) -> String {
    if bank.is_empty() {
        return String::new();
    }
    let mut state = seed.max(1);
    let mut result = Vec::with_capacity(count);
    let weights: Vec<u64> = bank
        .iter()
        .map(|(_, keys)| 1 + 4 * keys.iter().filter(|k| weak.contains(k)).count() as u64)
        .collect();
    let total: u64 = weights.iter().sum();
    for _ in 0..count {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let mut pick = state % total;
        for (i, weight) in weights.iter().enumerate() {
            if pick < *weight {
                result.push(bank[i].0.as_str());
                break;
            }
            pick -= weight;
        }
    }
    result.join(" ")
}
