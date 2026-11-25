pub mod blueprints;

use std::collections::HashSet;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::adaptive::required_keys_for_word;
use crate::corpus::Corpus;
pub use crate::curriculum::blueprints::CurriculumStage;
use crate::curriculum::blueprints::{UnitBlueprint, blueprints_for_layout};
use crate::keymap::LayoutKind;

pub const UNITS_PER_LAYOUT: u32 = 28;
pub const LESSONS_PER_UNIT: u32 = 25;
pub const TOTAL_LESSONS_PER_LAYOUT: u32 = UNITS_PER_LAYOUT * LESSONS_PER_UNIT;

/// Pedagogical activity type assigned to each lesson node in the 700-lesson Jungle Journey.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum LessonKind {
    KeyIntro,
    Drill,
    Words,
    Sentences,
    ReviewChallenge,
    MiniGame,
}

/// Complete generated lesson content returned when starting a Jungle lesson or Mini-Game.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct LessonDefinition {
    pub id: String,
    pub layout: LayoutKind,
    pub unit_number: u32,
    pub lesson_in_unit: u32,
    pub global_number: u32,
    pub stage: CurriculumStage,
    pub kind: LessonKind,
    pub title_en: String,
    pub title_hi: String,
    pub focus_keys: Vec<String>,
    pub target_wpm: f64,
    pub min_accuracy: f64,
    pub words: Vec<String>,
}

/// Compact node summary rendered inside the interactive Jungle Roadmap.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct LessonNodeSummary {
    pub id: String,
    pub unit_number: u32,
    pub lesson_in_unit: u32,
    pub global_number: u32,
    pub kind: LessonKind,
    pub title_en: String,
    pub title_hi: String,
    pub focus_keys: Vec<String>,
    pub target_wpm: f64,
    pub min_accuracy: f64,
    pub unlocked: bool,
    pub stars: u32,
    pub best_wpm: f64,
    pub best_accuracy: f64,
    pub completed: bool,
    pub attempts: u32,
}

/// Unit container grouping 25 sequential lesson nodes for the Jungle Map UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct UnitSummary {
    pub unit_number: u32,
    pub stage: CurriculumStage,
    pub title_en: String,
    pub title_hi: String,
    pub focus_keys: Vec<String>,
    pub target_wpm: f64,
    pub min_accuracy: f64,
    pub earned_stars: u32,
    pub max_stars: u32,
    pub completed_lessons: u32,
    pub unlocked: bool,
    pub lessons: Vec<LessonNodeSummary>,
}

/// Full Jungle Journey roadmap overview for a layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct JungleRoadmapSnapshot {
    pub layout: LayoutKind,
    pub total_earned_stars: u32,
    pub total_max_stars: u32,
    pub completed_lessons: u32,
    pub total_lessons: u32,
    pub next_lesson_id: String,
    pub units: Vec<UnitSummary>,
}

/// Persisted star & speed record for a single Jungle Journey lesson.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct LessonProgressRecord {
    pub layout: LayoutKind,
    pub lesson_id: String,
    pub stars: u32,
    pub best_wpm: f64,
    pub best_accuracy: f64,
    pub completed: bool,
    pub attempts: u32,
    pub updated_at: String,
}

/// Grading outcome emitted upon finishing a Jungle Journey lesson or Mini-Game.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct LessonCompletionResult {
    pub lesson_id: String,
    pub layout: LayoutKind,
    pub kind: LessonKind,
    pub stars_earned: u32,
    pub best_stars: u32,
    pub is_new_best: bool,
    pub passed: bool,
    pub net_wpm: f64,
    pub accuracy: f64,
    pub target_wpm: f64,
    pub min_accuracy: f64,
    pub next_lesson_id: Option<String>,
    pub next_lesson_unlocked: bool,
}

/// Computes canonical lesson identifier (`u01-l01` .. `u28-l25`).
pub fn lesson_id_for(unit_number: u32, lesson_in_unit: u32) -> String {
    format!("u{:02}-l{:02}", unit_number, lesson_in_unit)
}

/// Determines 0-to-5 star rating against the lesson's speed & accuracy targets.
/// Earning >= 3 stars unlocks the subsequent node on the Jungle Roadmap.
pub fn calculate_lesson_stars(
    net_wpm: f64,
    accuracy: f64,
    target_wpm: f64,
    min_accuracy: f64,
) -> u32 {
    if accuracy < (min_accuracy - 5.0).max(50.0) {
        return 0;
    }
    if accuracy >= 98.0 && net_wpm >= target_wpm * 1.5 {
        return 5;
    }
    if accuracy >= (min_accuracy + 2.0).min(97.0) && net_wpm >= target_wpm * 1.25 {
        return 4;
    }
    if accuracy >= min_accuracy && net_wpm >= target_wpm {
        return 3;
    }
    if accuracy >= min_accuracy && net_wpm >= target_wpm * 0.65 {
        return 2;
    }
    1
}

/// Determines lesson type within a 25-lesson unit so every 10th global lesson is a MiniGame
/// and every unit has a capstone ReviewChallenge.
pub fn lesson_kind_for(unit_number: u32, lesson_in_unit: u32) -> LessonKind {
    let global = (unit_number - 1) * LESSONS_PER_UNIT + lesson_in_unit;
    if global.is_multiple_of(10) {
        return LessonKind::MiniGame;
    }
    let unit_ends_on_tenth = (unit_number * LESSONS_PER_UNIT).is_multiple_of(10);
    if (!unit_ends_on_tenth && lesson_in_unit == LESSONS_PER_UNIT)
        || (unit_ends_on_tenth && lesson_in_unit == LESSONS_PER_UNIT - 1)
    {
        return LessonKind::ReviewChallenge;
    }

    match lesson_in_unit {
        1 | 11 => LessonKind::KeyIntro,
        2 | 3 | 4 | 8 | 12 | 13 | 14 => LessonKind::Drill,
        9 | 18 | 19 | 23 | 24 => LessonKind::Sentences,
        _ => LessonKind::Words,
    }
}

/// Generates all 700 lessons (28 units x 25 lessons) deterministically for `layout`.
pub fn generate_all_lessons(layout: LayoutKind, corpus: &Corpus) -> Vec<LessonDefinition> {
    let units = blueprints_for_layout(layout);
    let mut cumulative_keys: Vec<String> = Vec::new();
    let mut out = Vec::with_capacity(TOTAL_LESSONS_PER_LAYOUT as usize);

    for bp in units {
        for &k in bp.focus_keys {
            if !cumulative_keys.iter().any(|existing| existing == k) {
                cumulative_keys.push(k.to_string());
            }
        }
        for lesson_in_unit in 1..=LESSONS_PER_UNIT {
            out.push(build_lesson(
                layout,
                bp,
                lesson_in_unit,
                &cumulative_keys,
                corpus,
            ));
        }
    }
    out
}

/// Builds a single deterministic lesson by `lesson_id` (`u01-l01` .. `u28-l25`).
pub fn generate_lesson_by_id(
    layout: LayoutKind,
    lesson_id: &str,
    corpus: &Corpus,
) -> Option<LessonDefinition> {
    let (unit_number, lesson_in_unit) = parse_lesson_id(lesson_id)?;
    let units = blueprints_for_layout(layout);
    let bp = units.iter().find(|u| u.unit_number == unit_number)?;
    let mut cumulative_keys = Vec::new();
    for u in units.iter().take_while(|u| u.unit_number <= unit_number) {
        for &k in u.focus_keys {
            if !cumulative_keys.iter().any(|existing| existing == k) {
                cumulative_keys.push(k.to_string());
            }
        }
    }
    Some(build_lesson(
        layout,
        bp,
        lesson_in_unit,
        &cumulative_keys,
        corpus,
    ))
}

/// Assembles the full 28-unit Jungle Roadmap merged with persisted `LessonProgressRecord`s.
pub fn build_roadmap_snapshot(
    layout: LayoutKind,
    progress_rows: &[LessonProgressRecord],
) -> JungleRoadmapSnapshot {
    use std::collections::HashMap;
    let progress_map: HashMap<&str, &LessonProgressRecord> = progress_rows
        .iter()
        .map(|p| (p.lesson_id.as_str(), p))
        .collect();

    let units_bp = blueprints_for_layout(layout);
    let mut units = Vec::with_capacity(units_bp.len());
    let mut prev_passed = true;
    let mut total_earned_stars = 0u32;
    let mut completed_lessons = 0u32;
    let mut next_lesson_id: Option<String> = None;

    for bp in units_bp {
        let mut lessons = Vec::with_capacity(LESSONS_PER_UNIT as usize);
        let mut unit_stars = 0u32;
        let mut unit_completed = 0u32;
        let mut unit_unlocked = false;

        for lesson_in_unit in 1..=LESSONS_PER_UNIT {
            let id = lesson_id_for(bp.unit_number, lesson_in_unit);
            let global_number = (bp.unit_number - 1) * LESSONS_PER_UNIT + lesson_in_unit;
            let kind = lesson_kind_for(bp.unit_number, lesson_in_unit);
            let (title_en, title_hi) = lesson_titles(bp, lesson_in_unit, kind);
            let rec = progress_map.get(id.as_str()).copied();
            let stars = rec.map_or(0, |r| r.stars.min(5));
            let completed = rec.is_some_and(|r| r.completed);
            let unlocked = prev_passed;

            if unlocked {
                unit_unlocked = true;
            }
            if unlocked && stars < 3 && next_lesson_id.is_none() {
                next_lesson_id = Some(id.clone());
            }

            unit_stars += stars;
            if completed {
                unit_completed += 1;
                completed_lessons += 1;
            }
            prev_passed = stars >= 3;

            let progress_ratio = (lesson_in_unit as f64 - 1.0) / (LESSONS_PER_UNIT as f64);
            let target_wpm = (bp.target_wpm + progress_ratio * 2.0).round();

            lessons.push(LessonNodeSummary {
                id,
                unit_number: bp.unit_number,
                lesson_in_unit,
                global_number,
                kind,
                title_en,
                title_hi,
                focus_keys: bp.focus_keys.iter().map(|s| (*s).to_string()).collect(),
                target_wpm,
                min_accuracy: bp.min_accuracy,
                unlocked,
                stars,
                best_wpm: rec.map_or(0.0, |r| r.best_wpm),
                best_accuracy: rec.map_or(0.0, |r| r.best_accuracy),
                completed,
                attempts: rec.map_or(0, |r| r.attempts),
            });
        }

        total_earned_stars += unit_stars;
        units.push(UnitSummary {
            unit_number: bp.unit_number,
            stage: bp.stage,
            title_en: bp.title_en.to_string(),
            title_hi: bp.title_hi.to_string(),
            focus_keys: bp.focus_keys.iter().map(|s| (*s).to_string()).collect(),
            target_wpm: bp.target_wpm,
            min_accuracy: bp.min_accuracy,
            earned_stars: unit_stars,
            max_stars: LESSONS_PER_UNIT * 5,
            completed_lessons: unit_completed,
            unlocked: unit_unlocked,
            lessons,
        });
    }

    JungleRoadmapSnapshot {
        layout,
        total_earned_stars,
        total_max_stars: TOTAL_LESSONS_PER_LAYOUT * 5,
        completed_lessons,
        total_lessons: TOTAL_LESSONS_PER_LAYOUT,
        next_lesson_id: next_lesson_id.unwrap_or_else(|| lesson_id_for(1, 1)),
        units,
    }
}

pub fn next_lesson_id_after(lesson_id: &str) -> Option<String> {
    let (u, l) = parse_lesson_id(lesson_id)?;
    if l < LESSONS_PER_UNIT {
        Some(lesson_id_for(u, l + 1))
    } else if u < UNITS_PER_LAYOUT {
        Some(lesson_id_for(u + 1, 1))
    } else {
        None
    }
}

fn parse_lesson_id(id: &str) -> Option<(u32, u32)> {
    let rest = id.strip_prefix('u')?;
    let (u_str, l_str) = rest.split_once("-l")?;
    let u: u32 = u_str.parse().ok()?;
    let l: u32 = l_str.parse().ok()?;
    if (1..=UNITS_PER_LAYOUT).contains(&u) && (1..=LESSONS_PER_UNIT).contains(&l) {
        Some((u, l))
    } else {
        None
    }
}

fn build_lesson(
    layout: LayoutKind,
    bp: &UnitBlueprint,
    lesson_in_unit: u32,
    cumulative_keys: &[String],
    corpus: &Corpus,
) -> LessonDefinition {
    let id = lesson_id_for(bp.unit_number, lesson_in_unit);
    let global_number = (bp.unit_number - 1) * LESSONS_PER_UNIT + lesson_in_unit;
    let kind = lesson_kind_for(bp.unit_number, lesson_in_unit);
    let (title_en, title_hi) = lesson_titles(bp, lesson_in_unit, kind);

    let layout_seed = match layout {
        LayoutKind::EnQwerty => 1_000_003u64,
        LayoutKind::HiInscript => 2_000_003u64,
        LayoutKind::HiRemingtonGail => 3_000_003u64,
    };
    let seed = layout_seed ^ ((global_number as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut rng = StdRng::seed_from_u64(seed);

    let words = generate_lesson_words(
        layout,
        bp,
        kind,
        lesson_in_unit,
        cumulative_keys,
        corpus,
        &mut rng,
    );

    let progress_ratio = (lesson_in_unit as f64 - 1.0) / (LESSONS_PER_UNIT as f64);
    let target_wpm = (bp.target_wpm + progress_ratio * 2.0).round();

    LessonDefinition {
        id,
        layout,
        unit_number: bp.unit_number,
        lesson_in_unit,
        global_number,
        stage: bp.stage,
        kind,
        title_en,
        title_hi,
        focus_keys: bp.focus_keys.iter().map(|s| (*s).to_string()).collect(),
        target_wpm,
        min_accuracy: bp.min_accuracy,
        words,
    }
}

fn lesson_titles(bp: &UnitBlueprint, lesson_in_unit: u32, kind: LessonKind) -> (String, String) {
    let (en_tag, hi_tag) = match kind {
        LessonKind::KeyIntro => ("Key Introduction", "नई कुंजी परिचय"),
        LessonKind::Drill => ("Rhythm Drill", "लयबद्ध अभ्यास"),
        LessonKind::Words => ("Word Builder", "शब्द निर्माण"),
        LessonKind::Sentences => ("Sentence Flow", "वाक्य अभ्यास"),
        LessonKind::MiniGame => ("Balloon Pop Challenge", "बैलून पॉप गेम"),
        LessonKind::ReviewChallenge => ("Unit Boss Challenge", "इकाई दक्षता परीक्षा"),
    };
    (
        format!("{en_tag} · U{}.{}", bp.unit_number, lesson_in_unit),
        format!("{hi_tag} · {}.{}", bp.unit_number, lesson_in_unit),
    )
}

fn generate_lesson_words(
    layout: LayoutKind,
    bp: &UnitBlueprint,
    kind: LessonKind,
    lesson_in_unit: u32,
    cumulative_keys: &[String],
    corpus: &Corpus,
    rng: &mut impl Rng,
) -> Vec<String> {
    let unlocked_set: HashSet<&str> = cumulative_keys.iter().map(String::as_str).collect();
    let focus_set: HashSet<&str> = bp.focus_keys.iter().copied().collect();

    let matching_corpus: Vec<String> = corpus
        .words_for(layout)
        .iter()
        .filter_map(|w| {
            let req = required_keys_for_word(layout, w)?;
            if req.iter().all(|k| unlocked_set.contains(k.as_str())) {
                Some(w.clone())
            } else {
                None
            }
        })
        .collect();

    let focus_corpus: Vec<String> = corpus
        .words_for(layout)
        .iter()
        .filter_map(|w| {
            let req = required_keys_for_word(layout, w)?;
            if req.iter().all(|k| unlocked_set.contains(k.as_str()))
                && req.iter().any(|k| focus_set.contains(k.as_str()))
            {
                Some(w.clone())
            } else {
                None
            }
        })
        .collect();

    let phrase_words: Vec<String> = bp
        .sample_phrases
        .iter()
        .flat_map(|p| p.split_whitespace().map(str::to_string))
        .collect();

    match kind {
        LessonKind::KeyIntro => build_intro_patterns(layout, bp, lesson_in_unit),
        LessonKind::Drill => {
            let patterns = build_drill_patterns(layout, bp, cumulative_keys, rng);
            if focus_corpus.len() >= 4 && lesson_in_unit >= 8 {
                interleave_words(&patterns, &focus_corpus, 14, rng)
            } else {
                patterns
            }
        }
        LessonKind::Words => {
            let pool = if !focus_corpus.is_empty() {
                &focus_corpus
            } else if !matching_corpus.is_empty() {
                &matching_corpus
            } else {
                &phrase_words
            };
            sample_from_pool(pool, &matching_corpus, 16, rng)
        }
        LessonKind::Sentences | LessonKind::ReviewChallenge => {
            let count = if kind == LessonKind::ReviewChallenge { 20 } else { 15 };
            let phrase_idx = (lesson_in_unit as usize) % bp.sample_phrases.len();
            let mut out: Vec<String> = bp.sample_phrases[phrase_idx]
                .split_whitespace()
                .map(str::to_string)
                .collect();
            let extra_pool = if !matching_corpus.is_empty() {
                &matching_corpus
            } else {
                &phrase_words
            };
            while out.len() < count && !extra_pool.is_empty() {
                out.push(extra_pool[rng.random_range(0..extra_pool.len())].clone());
            }
            out
        }
        LessonKind::MiniGame => {
            let fallback = if !matching_corpus.is_empty() {
                matching_corpus
            } else if !phrase_words.is_empty() {
                phrase_words
            } else {
                build_intro_patterns(layout, bp, lesson_in_unit)
            };
            // Keep mini-game balloon words <= 8 graphemes for readable floating badges
            let compact: Vec<String> = fallback
                .iter()
                .filter(|w| w.chars().count() <= 10)
                .cloned()
                .collect();
            let pool = if compact.is_empty() { &fallback } else { &compact };
            sample_from_pool(pool, &fallback, 18, rng)
        }
    }
}

fn build_intro_patterns(layout: LayoutKind, bp: &UnitBlueprint, lesson_in_unit: u32) -> Vec<String> {
    let keys = bp.focus_keys;
    let primary = keys
        .get(((lesson_in_unit as usize) / 10) % keys.len())
        .copied()
        .unwrap_or(keys[0]);
    let secondary = keys.get((keys.len() > 1) as usize).copied().unwrap_or(primary);

    // Combine Hindi vowel signs / halant / nukta with safe base consonants so every intro token is valid Unicode.
    if layout != LayoutKind::EnQwerty {
        let base_a = if layout == LayoutKind::HiInscript { "क" } else { "क" };
        let base_b = if layout == LayoutKind::HiInscript { "र" } else { "र" };
        let token_a = safe_hindi_token(layout, primary, base_a);
        let token_b = safe_hindi_token(layout, secondary, base_b);
        return vec![
            token_a.clone(),
            token_a.clone(),
            token_b.clone(),
            token_b.clone(),
            format!("{token_a}{token_b}"),
            format!("{token_b}{token_a}"),
            token_a.clone(),
            token_b.clone(),
            format!("{token_a}{token_a}"),
            format!("{token_b}{token_b}"),
            token_a,
            token_b,
        ];
    }

    let a = primary;
    let b = secondary;
    vec![
        format!("{a}{a}{a}"),
        format!("{b}{b}{b}"),
        format!("{a}{b}{a}"),
        format!("{b}{a}{b}"),
        format!("{a}{a}{b}{b}"),
        format!("{b}{b}{a}{a}"),
        format!("{a}{b}{a}{b}"),
        format!("{b}{a}{b}{a}"),
        format!("{a}{a}{a}"),
        format!("{b}{b}{b}"),
        format!("{a}{b}"),
        format!("{b}{a}"),
    ]
}

fn build_drill_patterns(
    layout: LayoutKind,
    bp: &UnitBlueprint,
    cumulative_keys: &[String],
    rng: &mut impl Rng,
) -> Vec<String> {
    let mut out = Vec::with_capacity(14);
    let anchor = if layout == LayoutKind::EnQwerty {
        cumulative_keys.first().map(String::as_str).unwrap_or("f")
    } else {
        "क"
    };
    for idx in 0..14 {
        let k = bp.focus_keys[idx % bp.focus_keys.len()];
        let partner = bp.focus_keys[(idx + 1) % bp.focus_keys.len()];
        if layout == LayoutKind::EnQwerty {
            let a = if k.chars().all(|c| c.is_ascii_alphanumeric()) { k } else { anchor };
            let b = if partner.chars().all(|c| c.is_ascii_alphanumeric()) { partner } else { anchor };
            let pattern = match idx % 4 {
                0 => format!("{a}{b}{a}"),
                1 => format!("{a}{anchor}{b}"),
                2 => format!("{b}{a}{anchor}"),
                _ => format!("{a}{a}{b}{b}"),
            };
            out.push(pattern);
        } else {
            let t1 = safe_hindi_token(layout, k, "क");
            let t2 = safe_hindi_token(layout, partner, "र");
            let phrase_word = bp
                .sample_phrases
                .first()
                .and_then(|s| s.split_whitespace().nth(idx % 3))
                .map(str::to_string);
            if idx % 3 == 2
                && let Some(pw) = phrase_word
            {
                out.push(pw);
            } else if rng.random_bool(0.5) {
                out.push(format!("{t1}{t2}"));
            } else {
                out.push(format!("{t2}{t1}"));
            }
        }
    }
    out
}

fn safe_hindi_token(layout: LayoutKind, key: &str, base_consonant: &str) -> String {
    match key {
        "ा" | "ि" | "ी" | "ु" | "ू" | "े" | "ै" | "ो" | "ौ" | "ृ" | "ं" | "ँ" | "ः" | "़" => {
            format!("{base_consonant}{key}")
        }
        "्" => format!("{base_consonant}्त"),
        "्र" => "प्र".to_string(),
        "र्" => "कर्म".to_string(),
        "क्" | "त्" | "म्" | "स्" | "न्" | "थ्" | "भ्" | "श्" | "ध्" | "ख्" | "ष्" | "ण्" | "घ्" | "क्ष्" => {
            let full: String = key.chars().take_while(|&c| c != '\u{094D}').collect();
            if full == "क्ष" {
                "क्षमा".to_string()
            } else if layout == LayoutKind::HiRemingtonGail {
                format!("{full}र")
            } else {
                format!("{key}त")
            }
        }
        "। " | "।" | "," | ";" | "." | "(" | ")" | "-" | "?" => format!("{base_consonant}र{key}"),
        _ => key.to_string(),
    }
}

fn interleave_words(
    drills: &[String],
    vocab: &[String],
    target_len: usize,
    rng: &mut impl Rng,
) -> Vec<String> {
    let mut out = Vec::with_capacity(target_len);
    for i in 0..target_len {
        if i % 2 == 0 || vocab.is_empty() {
            out.push(drills[i % drills.len()].clone());
        } else {
            out.push(vocab[rng.random_range(0..vocab.len())].clone());
        }
    }
    out
}

fn sample_from_pool(
    primary: &[String],
    secondary: &[String],
    count: usize,
    rng: &mut impl Rng,
) -> Vec<String> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let pool = if !primary.is_empty() && (i % 3 != 2 || secondary.is_empty()) {
            primary
        } else if !secondary.is_empty() {
            secondary
        } else {
            primary
        };
        if pool.is_empty() {
            break;
        }
        let mut pick = &pool[rng.random_range(0..pool.len())];
        if pool.len() > 1 && out.last().is_some_and(|prev| prev == pick) {
            pick = &pool[(rng.random_range(1..pool.len())) % pool.len()];
        }
        out.push(pick.clone());
    }
    out
}
