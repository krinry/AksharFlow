use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// Visual state of an individual grapheme cluster in the typing stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum GraphemeStatus {
    Pending,
    Matched,
    Error,
}

/// Minimal per-grapheme status patch sent inside `KeyEventDelta`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct GraphemeUpdate {
    pub index: u32,
    pub status: GraphemeStatus,
}

/// Splits `text` into NFC-normalized Unicode grapheme clusters suitable for Devanagari and Latin.
pub fn split_graphemes(text: &str) -> Vec<String> {
    let normalized: String = text.nfc().collect();
    normalized
        .graphemes(true)
        .map(str::to_string)
        .collect()
}

/// Normalizes typed input into NFC so composed/decomposed forms compare identically.
pub fn normalize_nfc(text: &str) -> String {
    text.nfc().collect()
}

/// Evaluates how `typed` compares against a single `target` grapheme cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphemeMatch {
    Empty,
    Partial,
    Exact,
    Mismatch,
}

pub fn evaluate_grapheme(target: &str, typed: &str) -> GraphemeMatch {
    if typed.is_empty() {
        return GraphemeMatch::Empty;
    }
    let norm_target = normalize_nfc(target);
    let norm_typed = normalize_nfc(typed);
    if norm_typed == norm_target {
        GraphemeMatch::Exact
    } else if norm_target.starts_with(&norm_typed) {
        GraphemeMatch::Partial
    } else {
        GraphemeMatch::Mismatch
    }
}
