use crate::chars::{classify, is_base, Class};
use serde::{Deserialize, Serialize};

/// Half-open UTF-8 byte and Unicode scalar ranges (never UTF-16 offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cluster {
    pub start: usize,
    pub end: usize,
    pub char_start: usize,
    pub char_end: usize,
}

pub fn segment(text: &str) -> Vec<Cluster> {
    let mut input = text.char_indices().peekable();
    let mut result = Vec::with_capacity(text.len() / 3 + 1);
    let mut chars = 0;
    while let Some((start, base)) = input.next() {
        let char_start = chars;
        chars += 1;
        let mut end = start + base.len_utf8();
        if is_base(base) {
            for _ in 0..2 {
                let mut probe = input.clone();
                if probe
                    .next()
                    .is_some_and(|(_, c)| classify(c) == Class::Coeng)
                {
                    if let Some((i, c)) = probe
                        .next()
                        .filter(|(_, c)| classify(*c) == Class::Consonant)
                    {
                        input = probe;
                        end = i + c.len_utf8();
                        chars += 2;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            if input
                .peek()
                .is_some_and(|(_, c)| classify(*c) == Class::Register)
            {
                let (i, c) = input.next().unwrap();
                end = i + c.len_utf8();
                chars += 1;
            }
            for class in [Class::DependentVowel, Class::Sign] {
                while input.peek().is_some_and(|(_, c)| classify(*c) == class) {
                    let (i, c) = input.next().unwrap();
                    end = i + c.len_utf8();
                    chars += 1;
                }
            }
        }
        result.push(Cluster {
            start,
            end,
            char_start,
            char_end: chars,
        });
    }
    result
}
