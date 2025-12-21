//! IPA stuff.

/// IPA phoneme identifiers.
/// These represent linguistic units, not audio behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phoneme {
    Vowel(Vowel),
    Consonant(Consonant),
}

/// IPA vowel inventory (subset, expandable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vowel {
    /// Open back unrounded vowel (IPA: ɑ)
    OpenBackUnrounded,
}

impl Vowel {
    /// Canonical IPA symbol.
    pub fn ipa(&self) -> &'static str {
        match self {
            Vowel::OpenBackUnrounded => "ɑ",
        }
    }
}

/// IPA consonant inventory (placeholder).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Consonant {
    // later
}
