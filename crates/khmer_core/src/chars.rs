use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    Consonant,
    IndependentVowel,
    DependentVowel,
    Register,
    Sign,
    Coeng,
    Punctuation,
    Digit,
    LekAttak,
    Lunar,
    Zwsp,
    Zwnj,
    Zwj,
    Space,
    Other,
}

pub fn classify(c: char) -> Class {
    use Class::*;
    match c as u32 {
        0x1780..=0x17a2 => Consonant,
        0x17a3..=0x17b3 => IndependentVowel,
        0x17b6..=0x17c5 => DependentVowel,
        0x17c9 | 0x17ca => Register,
        0x17c6..=0x17d1 | 0x17d3 | 0x17dd => Sign,
        0x17d2 => Coeng,
        0x17d4..=0x17da | 0x17dc => Punctuation,
        0x17e0..=0x17e9 => Digit,
        0x17f0..=0x17f9 => LekAttak,
        0x19e0..=0x19ff => Lunar,
        0x200b => Zwsp,
        0x200c => Zwnj,
        0x200d => Zwj,
        0x20 | 0x09 | 0x0a | 0x0d => Space,
        _ => Other,
    }
}
pub fn is_base(c: char) -> bool {
    matches!(classify(c), Class::Consonant | Class::IndependentVowel)
}
pub fn deprecated(c: char) -> bool {
    matches!(
        c as u32,
        0x17a3 | 0x17a4 | 0x17b4 | 0x17b5 | 0x17d3 | 0x17d8
    )
}
