//! IPA stuff.

use std::fmt::Debug;

/// IPA phoneme identifiers.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phoneme {
    Vowel(Vowel),
    Consonant(Consonant),
    Space,
}

impl Phoneme {
    /// Canonical IPA symbol.
    pub fn ipa(&self) -> &'static str {
        match self {
            Phoneme::Vowel(v) => v.ipa(),
            Phoneme::Consonant(c) => c.ipa(),
            Phoneme::Space => " ",
        }
    }
}

impl Debug for Phoneme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Phoneme::Vowel(_) => "v:",
            Phoneme::Consonant(_) => "c:",
            Phoneme::Space => "_",
        })?;
        f.write_str(self.ipa())
    }
}

/// IPA vowel inventory (subset, expandable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vowel {
    /// Open back unrounded vowel (IPA: ɑ)
    OpenBackUnrounded,
    /// Close front unrounded vowel (IPA: i)
    CloseFrontUnrounded,
}

impl Vowel {
    /// Canonical IPA symbol.
    pub fn ipa(&self) -> &'static str {
        match self {
            Vowel::OpenBackUnrounded => "ɑ",
            Vowel::CloseFrontUnrounded => "i",
        }
    }
}

/// IPA consonant inventory (placeholder).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Consonant {}

impl Consonant {
    /// Canonical IPA symbol.
    pub fn ipa(&self) -> &'static str {
        match *self {}
    }
}
