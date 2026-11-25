use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::AksharError;
use crate::keymap::LayoutKind;

const CORPUS_EN_JSON: &str = include_str!("../../data/corpus-en.json");
const CORPUS_HI_JSON: &str = include_str!("../../data/corpus-hi.json");
const EXAM_PASSAGES_JSON: &str = include_str!("../../data/exam-passages.json");

/// Built-in or user-saved multi-paragraph passage for Timed Exam Mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct ExamPassage {
    pub id: String,
    pub language: String,
    pub title: String,
    pub category: String,
    pub content: String,
    #[serde(default)]
    pub is_custom: bool,
}

/// Offline dictionary corpus & exam passage library bundled inside `aksharflow-core`.
#[derive(Debug, Clone)]
pub struct Corpus {
    en_words: Vec<String>,
    hi_words: Vec<String>,
    exam_passages: Vec<ExamPassage>,
}

impl Corpus {
    pub fn load_embedded() -> Result<Self, AksharError> {
        let en_words: Vec<String> = serde_json::from_str(CORPUS_EN_JSON)?;
        let hi_words: Vec<String> = serde_json::from_str(CORPUS_HI_JSON)?;
        let exam_passages: Vec<ExamPassage> = serde_json::from_str(EXAM_PASSAGES_JSON)?;
        Ok(Self {
            en_words,
            hi_words,
            exam_passages,
        })
    }

    pub fn words_for(&self, layout: LayoutKind) -> &[String] {
        match layout {
            LayoutKind::EnQwerty => &self.en_words,
            LayoutKind::HiInscript | LayoutKind::HiRemingtonGail => &self.hi_words,
        }
    }

    pub fn exam_passages(&self) -> &[ExamPassage] {
        &self.exam_passages
    }

    /// Generates `count` space-separated words without immediate back-to-back repeats.
    pub fn sample_words(
        &self,
        layout: LayoutKind,
        count: usize,
        rng: &mut impl Rng,
    ) -> Result<Vec<String>, AksharError> {
        let pool = self.words_for(layout);
        if pool.is_empty() {
            return Err(AksharError::EmptyCorpus(layout.to_string()));
        }
        let mut selected = Vec::with_capacity(count);
        for _ in 0..count {
            let mut candidate = &pool[rng.random_range(0..pool.len())];
            if pool.len() > 1 && selected.last().is_some_and(|prev| prev == candidate) {
                candidate = &pool[(rng.random_range(1..pool.len())) % pool.len()];
            }
            selected.push(candidate.clone());
        }
        Ok(selected)
    }
}
