use crate::cluster::segment;

/// Accepted replacements are explicit: only two coengs, RO moves last.
/// Neither vowel decomposition nor arbitrary consonant reordering is accepted.
pub const EQUIVALENTS: &[(&str, &str)] = &[
    ("្រ្ក", "្ក្រ"),
    ("្រ្ខ", "្ខ្រ"),
    ("្រ្គ", "្គ្រ"),
    ("្រ្ឃ", "្ឃ្រ"),
    ("្រ្ង", "្ង្រ"),
    ("្រ្ច", "្ច្រ"),
    ("្រ្ឆ", "្ឆ្រ"),
    ("្រ្ជ", "្ជ្រ"),
    ("្រ្ឈ", "្ឈ្រ"),
    ("្រ្ញ", "្ញ្រ"),
    ("្រ្ដ", "្ដ្រ"),
    ("្រ្ឋ", "្ឋ្រ"),
    ("្រ្ឌ", "្ឌ្រ"),
    ("្រ្ឍ", "្ឍ្រ"),
    ("្រ្ណ", "្ណ្រ"),
    ("្រ្ត", "្ត្រ"),
    ("្រ្ថ", "្ថ្រ"),
    ("្រ្ទ", "្ទ្រ"),
    ("្រ្ធ", "្ធ្រ"),
    ("្រ្ន", "្ន្រ"),
    ("្រ្ប", "្ប្រ"),
    ("្រ្ផ", "្ផ្រ"),
    ("្រ្ព", "្ព្រ"),
    ("្រ្ភ", "្ភ្រ"),
    ("្រ្ម", "្ម្រ"),
    ("្រ្យ", "្យ្រ"),
    ("្រ្ល", "្ល្រ"),
    ("្រ្វ", "្វ្រ"),
    ("្រ្ឝ", "្ឝ្រ"),
    ("្រ្ឞ", "្ឞ្រ"),
    ("្រ្ស", "្ស្រ"),
    ("្រ្ហ", "្ហ្រ"),
    ("្រ្ឡ", "្ឡ្រ"),
    ("្រ្អ", "្អ្រ"),
];

pub fn canonical_cluster(text: &str) -> String {
    // Four coeng code points are contiguous directly after the base.
    let mut chars = text.char_indices();
    chars.next();
    if let Some((start, _)) = chars.next() {
        for &(alternative, canonical) in EQUIVALENTS {
            if text[start..].starts_with(alternative) {
                return format!(
                    "{}{}{}",
                    &text[..start],
                    canonical,
                    &text[start + alternative.len()..]
                );
            }
        }
    }
    text.to_string()
}
pub fn canonicalize(text: &str) -> String {
    segment(text)
        .iter()
        .map(|c| canonical_cluster(&text[c.start..c.end]))
        .collect()
}
pub fn equivalent(a: &str, b: &str, strict: bool) -> bool {
    if strict {
        a == b
    } else {
        canonicalize(a) == canonicalize(b)
    }
}
pub fn accepted_orders(text: &str, strict: bool) -> Vec<String> {
    if strict {
        return vec![text.to_string()];
    }
    let canonical = canonical_cluster(text);
    let mut result = vec![canonical.clone()];
    for &(alternative, ordered) in EQUIVALENTS {
        if let Some((start, _)) = canonical.char_indices().nth(1) {
            if canonical[start..].starts_with(ordered) {
                result.push(format!(
                    "{}{}{}",
                    &canonical[..start],
                    alternative,
                    &canonical[start + ordered.len()..]
                ));
                break;
            }
        }
    }
    result
}
