use std::collections::HashSet;
use std::path::Path;

use chrono::Utc;
use rand::rng;
use serde::{Deserialize, Serialize};

use crate::adaptive::{
    UnlockedKeyEvent, generate_adaptive_line, generate_weak_drill_line, primary_key_id_for_target,
    reachable_unlocked_keys, reconcile_key_stats, record_telemetry_and_check_unlock,
};
use crate::corpus::{Corpus, ExamPassage};
use crate::curriculum::{
    JungleRoadmapSnapshot, LessonCompletionResult, LessonDefinition, LessonKind,
    LessonProgressRecord, build_roadmap_snapshot, calculate_lesson_stars, generate_lesson_by_id,
    next_lesson_id_after,
};
use crate::error::AksharError;
use crate::grapheme::{GraphemeMatch, GraphemeStatus, GraphemeUpdate, evaluate_grapheme, split_graphemes};
use crate::keymap::remington_gail::RemingtonIme;
use crate::keymap::{
    KeyInputEvent, KeycapDescriptor, LayoutKind, MappedKeyAction, NextKeyHint, PhysicalKey,
    hint_for_grapheme, inscript, layout_keycaps, qwerty,
};
use crate::metrics::{LiveMetrics, MetricsTracker};
#[cfg(not(feature = "sqlite"))]
use crate::storage::MemoryStorage;
#[cfg(feature = "sqlite")]
use crate::storage::SqliteStorage;
use crate::storage::{
    AppSettings, BackupBundleV1, CustomPassage, KeyStatRecord, SessionRecord, StorageBackend,
};

const DEFAULT_WORDS_PER_LINE: usize = 18;
const EXAM_WORDS_PER_CHUNK: usize = 24;

/// Active training mode inside `SessionEngine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum PracticeMode {
    Adaptive,
    WeakDrill,
    Exam,
    Jungle,
    MiniGame,
}

impl PracticeMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adaptive => "adaptive",
            Self::WeakDrill => "weak_drill",
            Self::Exam => "exam",
            Self::Jungle => "jungle",
            Self::MiniGame => "mini_game",
        }
    }
}

/// Audio feedback cue attached to `KeyEventDelta` so UI sound playback requires zero parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum KeySoundKind {
    None,
    Tap,
    Error,
    Space,
    Backspace,
    LineComplete,
}

/// New line payload sent inside `KeyEventDelta` only when a line completes and rolls over.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct LineRolloverPayload {
    pub line_index: u32,
    pub words: Vec<String>,
    pub graphemes: Vec<String>,
}

/// Metadata attached to `SessionSnapshot` when running a Jungle Journey lesson or Mini-Game.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct ActiveLessonMeta {
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
}

/// Initial state snapshot sent once when starting or restarting a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct SessionSnapshot {
    pub session_id: String,
    pub layout: LayoutKind,
    pub mode: PracticeMode,
    pub allow_backspace: bool,
    pub exam_duration_secs: Option<u32>,
    pub lesson_meta: Option<ActiveLessonMeta>,
    pub line_index: u32,
    pub words: Vec<String>,
    pub graphemes: Vec<String>,
    pub keycaps: Vec<KeycapDescriptor>,
    pub delta: KeyEventDelta,
}

/// Full bootstrap payload sent on app launch, layout switch, backup import, or progress reset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct BootstrapSnapshot {
    pub settings: AppSettings,
    pub session: SessionSnapshot,
    pub key_stats: Vec<KeyStatRecord>,
    pub recent_sessions: Vec<SessionRecord>,
}

/// Aggregate analytics payload for a layout, including Jungle Journey star mastery.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct AnalyticsSnapshot {
    pub layout: LayoutKind,
    pub key_stats: Vec<KeyStatRecord>,
    pub recent_sessions: Vec<SessionRecord>,
    pub total_sessions: u32,
    pub avg_net_wpm: f64,
    pub best_net_wpm: f64,
    pub avg_accuracy: f64,
    pub unlocked_count: u32,
    pub total_keys: u32,
    pub jungle_earned_stars: u32,
    pub jungle_max_stars: u32,
    pub jungle_completed_lessons: u32,
    pub jungle_total_lessons: u32,
}

/// Summary returned after exporting a `.json` backup file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct ExportResult {
    pub path: String,
    pub exported_at: String,
    pub key_stats_count: u32,
    pub sessions_count: u32,
    pub passages_count: u32,
}

/// Lightweight per-keystroke state delta (Safeguard #1) to avoid re-serializing the full line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct KeyEventDelta {
    pub grapheme_index: u32,
    pub active_typed_preview: String,
    pub active_is_partial: bool,
    pub active_has_error: bool,
    pub committed_updates: Vec<GraphemeUpdate>,
    pub pending_ime_preview: Option<String>,
    pub next_key_hint: Option<NextKeyHint>,
    pub metrics: LiveMetrics,
    pub sound_kind: KeySoundKind,
    pub line_completed: bool,
    pub next_line: Option<LineRolloverPayload>,
    pub newly_unlocked_key: Option<UnlockedKeyEvent>,
    pub updated_key_stats: Option<Vec<KeyStatRecord>>,
}

/// Result returned by `resolve_game_keystroke` for the Falling Words / Balloon Pop Mini-Game.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct GameKeyResult {
    pub locked_word: Option<String>,
    pub typed_preview: String,
    pub matched_graphemes: u32,
    pub total_graphemes: u32,
    pub pending_ime_preview: Option<String>,
    pub popped: bool,
    pub is_error: bool,
    pub sound_kind: KeySoundKind,
    pub metrics: LiveMetrics,
}

/// Final summary returned when ending a session or exam.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct SessionSummary {
    pub session_id: String,
    pub layout: LayoutKind,
    pub mode: PracticeMode,
    pub lines_completed: u32,
    pub metrics: LiveMetrics,
}

#[derive(Debug)]
pub struct SessionEngine {
    corpus: Corpus,
    storage: Box<dyn StorageBackend>,
    settings: AppSettings,
    key_stats: Vec<KeyStatRecord>,
    reachable_keys: HashSet<String>,
    session_id: String,
    layout: LayoutKind,
    mode: PracticeMode,
    allow_backspace: bool,
    exam_duration_secs: Option<u32>,
    exam_words: Vec<String>,
    exam_cursor: usize,
    drill_focus_keys: Vec<String>,
    active_lesson: Option<LessonDefinition>,
    words_per_line: usize,
    line_index: u32,
    lines_completed: u32,
    words: Vec<String>,
    graphemes: Vec<String>,
    statuses: Vec<GraphemeStatus>,
    typed_slots: Vec<String>,
    grapheme_index: usize,
    active_typed: String,
    remington_ime: RemingtonIme,
    game_locked_word: Option<String>,
    game_graphemes: Vec<String>,
    game_grapheme_index: usize,
    game_active_typed: String,
    game_ime: RemingtonIme,
    metrics: MetricsTracker,
    last_key_ts_ms: Option<f64>,
    active: bool,
}

impl SessionEngine {
    /// Creates an in-memory engine (SQLite in-memory when `sqlite` feature is enabled, otherwise `MemoryStorage`).
    pub fn new() -> Result<Self, AksharError> {
        #[cfg(feature = "sqlite")]
        {
            let storage = SqliteStorage::open_in_memory()?;
            return Self::with_storage(storage);
        }
        #[cfg(not(feature = "sqlite"))]
        {
            Self::with_storage(MemoryStorage::new())
        }
    }

    /// Opens an on-disk SQLite database at `db_path` and initializes the engine.
    #[cfg(feature = "sqlite")]
    pub fn open_db(db_path: &Path) -> Result<Self, AksharError> {
        let storage = SqliteStorage::open(db_path)?;
        Self::with_storage(storage)
    }

    /// Initializes the engine around any [`StorageBackend`] (SQLite, in-memory, or browser storage).
    pub fn with_storage<S: StorageBackend + 'static>(storage: S) -> Result<Self, AksharError> {
        Self::with_boxed_storage(Box::new(storage))
    }

    /// Initializes the engine around a boxed [`StorageBackend`].
    pub fn with_boxed_storage(mut storage: Box<dyn StorageBackend>) -> Result<Self, AksharError> {
        let corpus = Corpus::load_embedded()?;
        let settings = storage.load_or_init_settings()?;
        let layout = settings.active_layout;
        let persisted_stats = storage.load_key_stats(layout)?;
        let key_stats = reconcile_key_stats(layout, persisted_stats);
        storage.upsert_key_stats(&key_stats)?;
        let reachable_keys = reachable_unlocked_keys(layout, &key_stats, &corpus);

        Ok(Self {
            corpus,
            storage,
            settings,
            key_stats,
            reachable_keys,
            session_id: String::new(),
            layout,
            mode: PracticeMode::Adaptive,
            allow_backspace: true,
            exam_duration_secs: None,
            exam_words: Vec::new(),
            exam_cursor: 0,
            drill_focus_keys: Vec::new(),
            active_lesson: None,
            words_per_line: DEFAULT_WORDS_PER_LINE,
            line_index: 0,
            lines_completed: 0,
            words: Vec::new(),
            graphemes: Vec::new(),
            statuses: Vec::new(),
            typed_slots: Vec::new(),
            grapheme_index: 0,
            active_typed: String::new(),
            remington_ime: RemingtonIme::new(),
            game_locked_word: None,
            game_graphemes: Vec::new(),
            game_grapheme_index: 0,
            game_active_typed: String::new(),
            game_ime: RemingtonIme::new(),
            metrics: MetricsTracker::new(),
            last_key_ts_ms: None,
            active: false,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn key_stats(&self) -> &[KeyStatRecord] {
        &self.key_stats
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Loads settings, starts an adaptive session for the persisted active layout, and returns full bootstrap state.
    pub fn bootstrap(&mut self, words_per_line: Option<usize>) -> Result<BootstrapSnapshot, AksharError> {
        let session = self.start_session(self.settings.active_layout, words_per_line)?;
        let recent_sessions = self.storage.list_typing_sessions(Some(self.layout), 30)?;
        Ok(BootstrapSnapshot {
            settings: self.settings.clone(),
            session,
            key_stats: self.key_stats.clone(),
            recent_sessions,
        })
    }

    /// Starts a fresh adaptive typing session for `layout` and persists `active_layout`.
    pub fn start_session(
        &mut self,
        layout: LayoutKind,
        words_per_line: Option<usize>,
    ) -> Result<SessionSnapshot, AksharError> {
        let count = words_per_line.unwrap_or(DEFAULT_WORDS_PER_LINE).clamp(5, 40);
        self.active_lesson = None;
        self.reset_session_shell(layout, PracticeMode::Adaptive, true, None, count)?;
        self.load_next_line_words(None)?;
        Ok(self.build_snapshot())
    }

    /// Starts a targeted drill session prioritizing `focus_key_ids` (triggered from Analytics).
    pub fn start_weak_key_drill(
        &mut self,
        layout: LayoutKind,
        focus_key_ids: Vec<String>,
        words_per_line: Option<usize>,
    ) -> Result<BootstrapSnapshot, AksharError> {
        let count = words_per_line.unwrap_or(DEFAULT_WORDS_PER_LINE).clamp(5, 40);
        self.drill_focus_keys = focus_key_ids;
        self.active_lesson = None;
        self.reset_session_shell(layout, PracticeMode::WeakDrill, true, None, count)?;
        self.load_next_line_words(None)?;
        let session = self.build_snapshot();
        let recent_sessions = self.storage.list_typing_sessions(Some(self.layout), 30)?;
        Ok(BootstrapSnapshot {
            settings: self.settings.clone(),
            session,
            key_stats: self.key_stats.clone(),
            recent_sessions,
        })
    }

    /// Starts a timed CPCT/SSC Exam Mode session over `passage_content`.
    pub fn start_exam_session(
        &mut self,
        layout: LayoutKind,
        passage_content: String,
        duration_secs: u32,
        allow_backspace: bool,
    ) -> Result<SessionSnapshot, AksharError> {
        let words: Vec<String> = passage_content
            .split_whitespace()
            .map(str::to_string)
            .collect();
        if words.is_empty() {
            return Err(AksharError::EmptyCorpus(layout.to_string()));
        }
        self.exam_words = words;
        self.exam_cursor = 0;
        self.active_lesson = None;
        self.reset_session_shell(
            layout,
            PracticeMode::Exam,
            allow_backspace,
            Some(duration_secs.clamp(30, 3600)),
            EXAM_WORDS_PER_CHUNK,
        )?;
        self.load_next_line_words(None)?;
        Ok(self.build_snapshot())
    }

    /// Returns the 28-unit / 700-lesson Jungle Roadmap merged with persisted star ratings.
    pub fn get_jungle_roadmap(&self, layout: LayoutKind) -> Result<JungleRoadmapSnapshot, AksharError> {
        let rows = self.storage.load_lesson_progress(layout)?;
        Ok(build_roadmap_snapshot(layout, &rows))
    }

    /// Starts a specific Jungle Journey lesson or Mini-Game wave by `lesson_id`.
    pub fn start_jungle_lesson(
        &mut self,
        layout: LayoutKind,
        lesson_id: &str,
    ) -> Result<SessionSnapshot, AksharError> {
        let def = generate_lesson_by_id(layout, lesson_id, &self.corpus)
            .ok_or_else(|| AksharError::EmptyCorpus(format!("{layout}:{lesson_id}")))?;
        let mode = if def.kind == LessonKind::MiniGame {
            PracticeMode::MiniGame
        } else {
            PracticeMode::Jungle
        };
        let words = def.words.clone();
        self.active_lesson = Some(def);
        self.reset_game_target();
        self.reset_session_shell(layout, mode, true, None, words.len())?;
        self.load_next_line_words(Some(words))?;
        Ok(self.build_snapshot())
    }

    /// Grades an active Jungle lesson or Mini-Game, persists 0-to-5 star progress, and unlocks the next node on >= 3 stars.
    pub fn complete_jungle_lesson(
        &mut self,
        override_wpm: Option<f64>,
        override_accuracy: Option<f64>,
        override_errors: Option<u32>,
        override_keystrokes: Option<u32>,
    ) -> Result<LessonCompletionResult, AksharError> {
        let def = self
            .active_lesson
            .clone()
            .ok_or(AksharError::SessionNotActive)?;
        self.active = false;

        let snap = self.metrics.snapshot(None);
        let net_wpm = override_wpm.unwrap_or(snap.net_wpm);
        let accuracy = override_accuracy.unwrap_or(snap.accuracy);
        let errors = override_errors.unwrap_or(snap.errors);
        let keystrokes = override_keystrokes.unwrap_or(snap.keystrokes);

        let stars_earned =
            calculate_lesson_stars(net_wpm, accuracy, def.target_wpm, def.min_accuracy);
        let passed = stars_earned >= 3;

        let existing = self
            .storage
            .load_lesson_progress(def.layout)?
            .into_iter()
            .find(|r| r.lesson_id == def.id);

        let prev_stars = existing.as_ref().map_or(0, |r| r.stars);
        let best_stars = prev_stars.max(stars_earned);
        let is_new_best = stars_earned > prev_stars;
        let updated = LessonProgressRecord {
            layout: def.layout,
            lesson_id: def.id.clone(),
            stars: best_stars,
            best_wpm: existing.as_ref().map_or(net_wpm, |r| r.best_wpm.max(net_wpm)),
            best_accuracy: existing.as_ref().map_or(accuracy, |r| r.best_accuracy.max(accuracy)),
            completed: existing.as_ref().is_some_and(|r| r.completed) || passed,
            attempts: existing.as_ref().map_or(1, |r| r.attempts + 1),
            updated_at: Utc::now().to_rfc3339(),
        };
        self.storage.upsert_lesson_progress(&updated)?;

        if keystrokes > 0 {
            let mut rec_snap = snap;
            rec_snap.net_wpm = net_wpm;
            rec_snap.accuracy = accuracy;
            rec_snap.errors = errors;
            rec_snap.keystrokes = keystrokes;
            self.persist_session_record(&rec_snap, "jungle")?;
        }

        let next_lesson_id = next_lesson_id_after(&def.id);
        let next_lesson_unlocked = passed && next_lesson_id.is_some();

        Ok(LessonCompletionResult {
            lesson_id: def.id,
            layout: def.layout,
            kind: def.kind,
            stars_earned,
            best_stars,
            is_new_best,
            passed,
            net_wpm,
            accuracy,
            target_wpm: def.target_wpm,
            min_accuracy: def.min_accuracy,
            next_lesson_id,
            next_lesson_unlocked,
        })
    }

    /// Routes Canvas Mini-Game keystrokes through the active layout's Rust IME & grapheme matcher.
    pub fn resolve_game_keystroke(
        &mut self,
        event: &KeyInputEvent,
        candidate_words: Vec<String>,
    ) -> Result<GameKeyResult, AksharError> {
        if !self.active {
            return Err(AksharError::SessionNotActive);
        }

        if event.code == "Escape" {
            self.reset_game_target();
            return Ok(self.game_result(false, false, KeySoundKind::None, event.timestamp_ms));
        }

        let Some(key) = PhysicalKey::from_dom_code(&event.code) else {
            return Err(AksharError::UnsupportedKeyCode(event.code.clone()));
        };

        // Space clears any dangling Remington `pending_short_i` and releases the current balloon lock.
        if key == PhysicalKey::Space {
            self.game_ime.reset();
            self.reset_game_target();
            return Ok(self.game_result(false, false, KeySoundKind::Space, event.timestamp_ms));
        }

        if key == PhysicalKey::Backspace {
            if self.layout == LayoutKind::HiRemingtonGail {
                if let Some(prev) = self.game_ime.undo_step() {
                    self.game_active_typed = prev;
                } else if !self.game_active_typed.is_empty() {
                    self.game_active_typed.pop();
                } else if self.game_grapheme_index > 0 {
                    self.game_grapheme_index -= 1;
                } else {
                    self.reset_game_target();
                }
            } else if !self.game_active_typed.is_empty() {
                self.game_active_typed.pop();
            } else if self.game_grapheme_index > 0 {
                self.game_grapheme_index -= 1;
            } else {
                self.reset_game_target();
            }
            return Ok(self.game_result(false, false, KeySoundKind::Backspace, event.timestamp_ms));
        }

        // Auto-lock onto the lowest matching candidate balloon when no word is currently locked.
        if self.game_locked_word.is_none() {
            let mut matched_candidate: Option<String> = None;
            for word in candidate_words {
                let graphemes = split_graphemes(&word);
                if let Some(first_g) = graphemes.first() {
                    let mut probe_ime = RemingtonIme::new();
                    let (act, buf) =
                        map_with_layout(self.layout, key, event, "", &mut probe_ime);
                    let matches_first = match act {
                        MappedKeyAction::PendingPreview(_) => {
                            self.layout == LayoutKind::HiRemingtonGail
                                && first_g.ends_with('\u{093F}')
                        }
                        MappedKeyAction::NoOp => false,
                        _ => matches!(
                            evaluate_grapheme(first_g, &buf),
                            GraphemeMatch::Exact | GraphemeMatch::Partial
                        ),
                    };
                    if matches_first {
                        matched_candidate = Some(word);
                        break;
                    }
                }
            }

            if let Some(word) = matched_candidate {
                self.game_locked_word = Some(word.clone());
                self.game_graphemes = split_graphemes(&word);
                self.game_grapheme_index = 0;
                self.game_active_typed.clear();
                self.game_ime.reset();
            } else {
                self.metrics.record_keystroke(event.timestamp_ms, false);
                return Ok(self.game_result(false, true, KeySoundKind::Error, event.timestamp_ms));
            }
        }

        let (action, updated) = map_with_layout(
            self.layout,
            key,
            event,
            &self.game_active_typed,
            &mut self.game_ime,
        );

        if matches!(action, MappedKeyAction::NoOp) {
            return Ok(self.game_result(false, false, KeySoundKind::None, event.timestamp_ms));
        }

        let target = self
            .game_graphemes
            .get(self.game_grapheme_index)
            .cloned()
            .unwrap_or_default();

        if matches!(action, MappedKeyAction::PendingPreview(_)) {
            let valid = self.layout == LayoutKind::HiRemingtonGail && target.ends_with('\u{093F}');
            self.metrics.record_keystroke(event.timestamp_ms, valid);
            return Ok(self.game_result(
                false,
                !valid,
                if valid { KeySoundKind::Tap } else { KeySoundKind::Error },
                event.timestamp_ms,
            ));
        }

        self.game_active_typed = updated;
        match evaluate_grapheme(&target, &self.game_active_typed) {
            GraphemeMatch::Exact => {
                self.metrics.record_keystroke(event.timestamp_ms, true);
                self.game_active_typed.clear();
                self.game_ime.reset();
                self.game_grapheme_index += 1;
                if self.game_grapheme_index >= self.game_graphemes.len() {
                    let result =
                        self.game_result(true, false, KeySoundKind::LineComplete, event.timestamp_ms);
                    self.reset_game_target();
                    return Ok(result);
                }
                Ok(self.game_result(false, false, KeySoundKind::Tap, event.timestamp_ms))
            }
            GraphemeMatch::Partial => {
                self.metrics.record_keystroke(event.timestamp_ms, true);
                Ok(self.game_result(false, false, KeySoundKind::Tap, event.timestamp_ms))
            }
            GraphemeMatch::Empty => {
                Ok(self.game_result(false, false, KeySoundKind::None, event.timestamp_ms))
            }
            GraphemeMatch::Mismatch => {
                self.metrics.record_keystroke(event.timestamp_ms, false);
                if target.chars().count() <= 1 {
                    self.game_active_typed.clear();
                    self.game_ime.reset();
                }
                Ok(self.game_result(false, true, KeySoundKind::Error, event.timestamp_ms))
            }
        }
    }

    /// Starts a session with explicit words (for deterministic tests or custom lines).
    pub fn start_with_words(
        &mut self,
        layout: LayoutKind,
        words: Vec<String>,
    ) -> Result<SessionSnapshot, AksharError> {
        if words.is_empty() {
            return Err(AksharError::EmptyCorpus(layout.to_string()));
        }
        self.active_lesson = None;
        self.reset_session_shell(layout, PracticeMode::Adaptive, true, None, words.len())?;
        self.load_next_line_words(Some(words))?;
        Ok(self.build_snapshot())
    }

    fn reset_game_target(&mut self) {
        self.game_locked_word = None;
        self.game_graphemes.clear();
        self.game_grapheme_index = 0;
        self.game_active_typed.clear();
        self.game_ime.reset();
    }

    fn game_result(
        &self,
        popped: bool,
        is_error: bool,
        sound_kind: KeySoundKind,
        timestamp_ms: f64,
    ) -> GameKeyResult {
        let completed_prefix: String = self
            .game_graphemes
            .iter()
            .take(self.game_grapheme_index)
            .cloned()
            .collect();
        let typed_preview = format!("{}{}", completed_prefix, self.game_active_typed);
        let pending = if self.layout == LayoutKind::HiRemingtonGail {
            self.game_ime.pending_preview().map(str::to_string)
        } else {
            None
        };
        GameKeyResult {
            locked_word: self.game_locked_word.clone(),
            typed_preview,
            matched_graphemes: self.game_grapheme_index as u32,
            total_graphemes: self.game_graphemes.len() as u32,
            pending_ime_preview: pending,
            popped,
            is_error,
            sound_kind,
            metrics: self.metrics.snapshot(Some(timestamp_ms)),
        }
    }

    fn reset_session_shell(
        &mut self,
        layout: LayoutKind,
        mode: PracticeMode,
        allow_backspace: bool,
        exam_duration_secs: Option<u32>,
        words_per_line: usize,
    ) -> Result<(), AksharError> {
        if self.settings.active_layout != layout {
            self.settings.active_layout = layout;
            self.settings = self.storage.save_settings(&self.settings)?;
        }
        self.layout = layout;
        self.mode = mode;
        self.allow_backspace = allow_backspace;
        self.exam_duration_secs = exam_duration_secs;
        self.words_per_line = words_per_line;
        self.line_index = 0;
        self.lines_completed = 0;
        self.session_id = format!("af-{}", Utc::now().timestamp_millis());
        self.metrics.reset();
        self.last_key_ts_ms = None;
        self.active = true;

        let persisted = self.storage.load_key_stats(layout)?;
        self.key_stats = reconcile_key_stats(layout, persisted);
        self.storage.upsert_key_stats(&self.key_stats)?;
        self.reachable_keys = reachable_unlocked_keys(layout, &self.key_stats, &self.corpus);
        Ok(())
    }

    fn build_snapshot(&self) -> SessionSnapshot {
        let delta = self.build_delta(Vec::new(), KeySoundKind::None, false, None, None, None, None);
        let lesson_meta = self.active_lesson.as_ref().map(|d| ActiveLessonMeta {
            id: d.id.clone(),
            unit_number: d.unit_number,
            lesson_in_unit: d.lesson_in_unit,
            global_number: d.global_number,
            kind: d.kind,
            title_en: d.title_en.clone(),
            title_hi: d.title_hi.clone(),
            focus_keys: d.focus_keys.clone(),
            target_wpm: d.target_wpm,
            min_accuracy: d.min_accuracy,
        });
        SessionSnapshot {
            session_id: self.session_id.clone(),
            layout: self.layout,
            mode: self.mode,
            allow_backspace: self.allow_backspace,
            exam_duration_secs: self.exam_duration_secs,
            lesson_meta,
            line_index: self.line_index,
            words: self.words.clone(),
            graphemes: self.graphemes.clone(),
            keycaps: layout_keycaps(self.layout),
            delta,
        }
    }

    /// Processes a single hardware or virtual-keyboard key event and returns only a lightweight delta.
    pub fn process_key(&mut self, event: &KeyInputEvent) -> Result<KeyEventDelta, AksharError> {
        if !self.active {
            return Err(AksharError::SessionNotActive);
        }
        let Some(key) = PhysicalKey::from_dom_code(&event.code) else {
            return Err(AksharError::UnsupportedKeyCode(event.code.clone()));
        };

        if key == PhysicalKey::Backspace {
            if !self.allow_backspace {
                return Ok(self.build_delta(
                    Vec::new(),
                    KeySoundKind::Error,
                    false,
                    None,
                    None,
                    None,
                    Some(event.timestamp_ms),
                ));
            }
            return Ok(self.handle_backspace(event.ctrl, event.timestamp_ms));
        }

        if key == PhysicalKey::Space {
            return self.handle_space(event.timestamp_ms);
        }

        let latency_ms = self
            .last_key_ts_ms
            .map(|prev| (event.timestamp_ms - prev).max(0.0));
        self.last_key_ts_ms = Some(event.timestamp_ms);

        let expected_key_id = self.expected_telemetry_key_id();

        let (action, updated_buffer) = map_with_layout(
            self.layout,
            key,
            event,
            &self.active_typed,
            &mut self.remington_ime,
        );

        if matches!(action, MappedKeyAction::NoOp) {
            return Ok(self.build_delta(
                Vec::new(),
                KeySoundKind::None,
                false,
                None,
                None,
                None,
                Some(event.timestamp_ms),
            ));
        }

        if matches!(action, MappedKeyAction::PendingPreview(_)) {
            let target = self
                .graphemes
                .get(self.grapheme_index)
                .map(String::as_str)
                .unwrap_or("");
            let is_expected_pending = self.layout == LayoutKind::HiRemingtonGail
                && target.ends_with('\u{093F}');
            self.metrics
                .record_keystroke(event.timestamp_ms, is_expected_pending);
            let unlocked_evt = self.record_key_telemetry(
                expected_key_id.as_deref(),
                latency_ms,
                is_expected_pending,
            );
            let sound = if is_expected_pending {
                KeySoundKind::Tap
            } else {
                KeySoundKind::Error
            };
            let updated_stats = if unlocked_evt.is_some() {
                self.storage.upsert_key_stats(&self.key_stats)?;
                self.reachable_keys =
                    reachable_unlocked_keys(self.layout, &self.key_stats, &self.corpus);
                Some(self.key_stats.clone())
            } else {
                None
            };
            return Ok(self.build_delta(
                Vec::new(),
                sound,
                false,
                None,
                unlocked_evt,
                updated_stats,
                Some(event.timestamp_ms),
            ));
        }

        self.active_typed = updated_buffer;
        let mut committed = Vec::new();
        let target = self
            .graphemes
            .get(self.grapheme_index)
            .cloned()
            .unwrap_or_default();

        let eval = evaluate_grapheme(&target, &self.active_typed);
        let is_multi_codepoint_target = target.chars().count() > 1;
        let mut sound = KeySoundKind::Tap;
        let mut is_correct_stroke = true;

        match eval {
            GraphemeMatch::Exact => {
                self.metrics.record_keystroke(event.timestamp_ms, true);
                let idx = self.grapheme_index;
                self.statuses[idx] = GraphemeStatus::Matched;
                self.typed_slots[idx] = std::mem::take(&mut self.active_typed);
                committed.push(GraphemeUpdate {
                    index: idx as u32,
                    status: GraphemeStatus::Matched,
                });
                self.remington_ime.reset();
                self.grapheme_index += 1;
            }
            GraphemeMatch::Partial => {
                self.metrics.record_keystroke(event.timestamp_ms, true);
            }
            GraphemeMatch::Empty => {}
            GraphemeMatch::Mismatch => {
                is_correct_stroke = false;
                self.metrics.record_keystroke(event.timestamp_ms, false);
                sound = KeySoundKind::Error;

                if target != " " && !is_multi_codepoint_target {
                    let idx = self.grapheme_index;
                    self.statuses[idx] = GraphemeStatus::Error;
                    self.typed_slots[idx] = std::mem::take(&mut self.active_typed);
                    committed.push(GraphemeUpdate {
                        index: idx as u32,
                        status: GraphemeStatus::Error,
                    });
                    self.remington_ime.reset();
                    self.grapheme_index += 1;
                }
            }
        }

        let newly_unlocked = self.record_key_telemetry(
            expected_key_id.as_deref(),
            latency_ms,
            is_correct_stroke,
        );
        self.recount_uncorrected_errors();

        let (line_completed, next_line) = self.maybe_roll_line(event.timestamp_ms)?;
        if line_completed {
            sound = KeySoundKind::LineComplete;
        }
        let updated_stats = if (line_completed || newly_unlocked.is_some())
            && matches!(self.mode, PracticeMode::Adaptive | PracticeMode::WeakDrill)
        {
            self.storage.upsert_key_stats(&self.key_stats)?;
            if newly_unlocked.is_some() {
                self.reachable_keys =
                    reachable_unlocked_keys(self.layout, &self.key_stats, &self.corpus);
            }
            Some(self.key_stats.clone())
        } else {
            None
        };

        Ok(self.build_delta(
            committed,
            sound,
            line_completed,
            next_line,
            newly_unlocked,
            updated_stats,
            Some(event.timestamp_ms),
        ))
    }

    pub fn finish_session(&mut self) -> Result<SessionSummary, AksharError> {
        if !self.active {
            return Err(AksharError::SessionNotActive);
        }
        self.active = false;
        if matches!(self.mode, PracticeMode::Adaptive | PracticeMode::WeakDrill) {
            self.storage.upsert_key_stats(&self.key_stats)?;
        }
        let snapshot = self.metrics.snapshot(None);
        if snapshot.keystrokes > 0 {
            self.persist_session_record(&snapshot, "final")?;
        }
        Ok(SessionSummary {
            session_id: self.session_id.clone(),
            layout: self.layout,
            mode: self.mode,
            lines_completed: self.lines_completed,
            metrics: snapshot,
        })
    }

    pub fn update_settings(&mut self, settings: AppSettings) -> Result<AppSettings, AksharError> {
        let saved = self.storage.save_settings(&settings)?;
        self.settings = saved.clone();
        for stat in &mut self.key_stats {
            crate::adaptive::recompute_scores(stat, &self.settings);
        }
        self.storage.upsert_key_stats(&self.key_stats)?;
        Ok(saved)
    }

    pub fn analytics(&self, layout: LayoutKind) -> Result<AnalyticsSnapshot, AksharError> {
        let stats = if layout == self.layout {
            self.key_stats.clone()
        } else {
            reconcile_key_stats(layout, self.storage.load_key_stats(layout)?)
        };
        let recent_sessions = self.storage.list_typing_sessions(Some(layout), 50)?;
        let total_sessions = recent_sessions.len() as u32;
        let (avg_net_wpm, best_net_wpm, avg_accuracy) = if recent_sessions.is_empty() {
            (0.0, 0.0, 100.0)
        } else {
            let sum_wpm: f64 = recent_sessions.iter().map(|s| s.net_wpm).sum();
            let best: f64 = recent_sessions
                .iter()
                .map(|s| s.net_wpm)
                .fold(0.0, f64::max);
            let sum_acc: f64 = recent_sessions.iter().map(|s| s.accuracy).sum();
            let n = recent_sessions.len() as f64;
            (
                (sum_wpm / n * 10.0).round() / 10.0,
                (best * 10.0).round() / 10.0,
                (sum_acc / n * 10.0).round() / 10.0,
            )
        };
        let unlocked_count = stats.iter().filter(|s| s.unlocked).count() as u32;
        let total_keys = stats.len() as u32;
        let roadmap = self.get_jungle_roadmap(layout)?;

        Ok(AnalyticsSnapshot {
            layout,
            key_stats: stats,
            recent_sessions,
            total_sessions,
            avg_net_wpm,
            best_net_wpm,
            avg_accuracy,
            unlocked_count,
            total_keys,
            jungle_earned_stars: roadmap.total_earned_stars,
            jungle_max_stars: roadmap.total_max_stars,
            jungle_completed_lessons: roadmap.completed_lessons,
            jungle_total_lessons: roadmap.total_lessons,
        })
    }

    /// Returns built-in CPCT/SSC passages combined with any user-saved custom passages.
    pub fn list_exam_passages(&self) -> Result<Vec<ExamPassage>, AksharError> {
        let mut all: Vec<ExamPassage> = self.corpus.exam_passages().to_vec();
        for custom in self.storage.list_custom_passages()? {
            all.push(ExamPassage {
                id: custom.id,
                language: custom.language,
                title: custom.title,
                category: "Custom".to_string(),
                content: custom.content,
                is_custom: true,
            });
        }
        Ok(all)
    }

    pub fn export_backup(&self) -> Result<BackupBundleV1, AksharError> {
        self.storage.export_backup()
    }

    pub fn export_backup_json(&self) -> Result<String, AksharError> {
        let bundle = self.storage.export_backup()?;
        Ok(serde_json::to_string_pretty(&bundle)?)
    }

    pub fn import_backup_json(
        &mut self,
        raw_json: &str,
        replace: bool,
    ) -> Result<BootstrapSnapshot, AksharError> {
        let bundle: BackupBundleV1 = serde_json::from_str(raw_json)?;
        self.storage.import_backup(&bundle, replace)?;
        self.settings = self.storage.load_or_init_settings()?;
        self.bootstrap(Some(self.words_per_line))
    }

    pub fn export_backup_to_file(&self, path: &Path) -> Result<ExportResult, AksharError> {
        let bundle = self.storage.export_backup()?;
        let json = serde_json::to_string_pretty(&bundle)?;
        std::fs::write(path, json)?;
        Ok(ExportResult {
            path: path.to_string_lossy().to_string(),
            exported_at: bundle.exported_at,
            key_stats_count: bundle.key_stats.len() as u32,
            sessions_count: bundle.typing_sessions.len() as u32,
            passages_count: bundle.custom_passages.len() as u32,
        })
    }

    pub fn import_backup_from_file(
        &mut self,
        path: &Path,
        replace: bool,
    ) -> Result<BootstrapSnapshot, AksharError> {
        let raw = std::fs::read_to_string(path)?;
        self.import_backup_json(&raw, replace)
    }

    pub fn reset_layout_progress(&mut self, layout: LayoutKind) -> Result<BootstrapSnapshot, AksharError> {
        self.storage.delete_key_stats_for_layout(layout)?;
        let fresh = reconcile_key_stats(layout, Vec::new());
        self.storage.upsert_key_stats(&fresh)?;
        if layout == self.layout {
            self.key_stats = fresh;
        }
        self.start_session(layout, Some(self.words_per_line))?;
        self.bootstrap(Some(self.words_per_line))
    }

    pub fn save_custom_passage(
        &mut self,
        title: String,
        language: String,
        content: String,
    ) -> Result<CustomPassage, AksharError> {
        let passage = CustomPassage {
            id: format!("psg-{}", Utc::now().timestamp_millis()),
            language,
            title,
            content,
            created_at: Utc::now().to_rfc3339(),
        };
        self.storage.save_custom_passage(&passage)?;
        Ok(passage)
    }

    pub fn list_custom_passages(&self) -> Result<Vec<CustomPassage>, AksharError> {
        self.storage.list_custom_passages()
    }

    fn expected_telemetry_key_id(&self) -> Option<String> {
        let target = self
            .graphemes
            .get(self.grapheme_index)
            .map(String::as_str)
            .unwrap_or("");
        if target.is_empty() || target == " " {
            return None;
        }
        let pending = if self.layout == LayoutKind::HiRemingtonGail {
            self.remington_ime.pending_preview()
        } else {
            None
        };
        if let Some(hint) = hint_for_grapheme(self.layout, target, &self.active_typed, pending) {
            let label = if self.layout == LayoutKind::EnQwerty {
                hint.expected_label.to_ascii_lowercase()
            } else {
                hint.expected_label
            };
            if self.key_stats.iter().any(|k| k.key_id == label) {
                return Some(label);
            }
        }
        primary_key_id_for_target(self.layout, target)
    }

    fn record_key_telemetry(
        &mut self,
        expected_key_id: Option<&str>,
        latency_ms: Option<f64>,
        is_correct: bool,
    ) -> Option<UnlockedKeyEvent> {
        // Keep Smart Adaptive unlock progression isolated from Exam, Jungle, and Mini-Game runs.
        if !matches!(self.mode, PracticeMode::Adaptive | PracticeMode::WeakDrill) {
            return None;
        }
        let key_id = expected_key_id?;
        record_telemetry_and_check_unlock(
            &mut self.key_stats,
            key_id,
            latency_ms,
            is_correct,
            &self.settings,
            Some(&self.reachable_keys),
        )
    }

    fn handle_space(&mut self, timestamp_ms: f64) -> Result<KeyEventDelta, AksharError> {
        // Safeguard #3: Space safely flushes/clears any dangling `pending_short_i`.
        self.remington_ime.reset();
        self.last_key_ts_ms = Some(timestamp_ms);

        let mut committed = Vec::new();
        let target = self
            .graphemes
            .get(self.grapheme_index)
            .map(String::as_str)
            .unwrap_or("");

        let is_expected_space = target == " " && self.active_typed.is_empty();
        self.metrics.record_keystroke(timestamp_ms, is_expected_space);

        if target == " " {
            let idx = self.grapheme_index;
            let status = if self.active_typed.is_empty() {
                GraphemeStatus::Matched
            } else {
                GraphemeStatus::Error
            };
            self.statuses[idx] = status;
            self.typed_slots[idx] = " ".to_string();
            self.active_typed.clear();
            committed.push(GraphemeUpdate {
                index: idx as u32,
                status,
            });
            self.grapheme_index += 1;
        } else {
            while self.grapheme_index < self.graphemes.len() {
                let idx = self.grapheme_index;
                if self.graphemes[idx] == " " {
                    self.statuses[idx] = GraphemeStatus::Matched;
                    self.typed_slots[idx] = " ".to_string();
                    committed.push(GraphemeUpdate {
                        index: idx as u32,
                        status: GraphemeStatus::Matched,
                    });
                    self.grapheme_index += 1;
                    break;
                }
                self.statuses[idx] = GraphemeStatus::Error;
                if !self.active_typed.is_empty() {
                    self.typed_slots[idx] = std::mem::take(&mut self.active_typed);
                }
                committed.push(GraphemeUpdate {
                    index: idx as u32,
                    status: GraphemeStatus::Error,
                });
                self.grapheme_index += 1;
            }
            self.active_typed.clear();
        }

        if matches!(self.mode, PracticeMode::Adaptive | PracticeMode::WeakDrill) {
            self.storage.upsert_key_stats(&self.key_stats)?;
        }
        self.recount_uncorrected_errors();
        let (line_completed, next_line) = self.maybe_roll_line(timestamp_ms)?;
        let sound = if line_completed {
            KeySoundKind::LineComplete
        } else if is_expected_space {
            KeySoundKind::Space
        } else {
            KeySoundKind::Error
        };
        let updated_stats = if line_completed
            && matches!(self.mode, PracticeMode::Adaptive | PracticeMode::WeakDrill)
        {
            Some(self.key_stats.clone())
        } else {
            None
        };

        Ok(self.build_delta(
            committed,
            sound,
            line_completed,
            next_line,
            None,
            updated_stats,
            Some(timestamp_ms),
        ))
    }

    fn handle_backspace(&mut self, ctrl: bool, timestamp_ms: f64) -> KeyEventDelta {
        let mut committed = Vec::new();

        if self.layout == LayoutKind::HiRemingtonGail {
            if ctrl {
                self.remington_ime.reset();
                if !self.active_typed.is_empty() {
                    self.active_typed.clear();
                } else if self.grapheme_index > 0 {
                    self.step_back_slot(&mut committed, true);
                }
            } else if let Some(prev_buf) = self.remington_ime.undo_step() {
                self.active_typed = prev_buf;
            } else if !self.active_typed.is_empty() {
                self.active_typed.pop();
            } else if self.grapheme_index > 0 {
                self.step_back_slot(&mut committed, false);
            }
        } else if ctrl {
            if !self.active_typed.is_empty() {
                self.active_typed.clear();
            } else if self.grapheme_index > 0 {
                self.step_back_slot(&mut committed, true);
            }
        } else if !self.active_typed.is_empty() {
            self.active_typed.pop();
        } else if self.grapheme_index > 0 {
            self.step_back_slot(&mut committed, false);
        }

        self.recount_uncorrected_errors();
        self.build_delta(
            committed,
            KeySoundKind::Backspace,
            false,
            None,
            None,
            None,
            Some(timestamp_ms),
        )
    }

    fn step_back_slot(&mut self, committed: &mut Vec<GraphemeUpdate>, clear_whole: bool) {
        self.grapheme_index -= 1;
        let idx = self.grapheme_index;
        self.statuses[idx] = GraphemeStatus::Pending;
        committed.push(GraphemeUpdate {
            index: idx as u32,
            status: GraphemeStatus::Pending,
        });
        let prev_typed = std::mem::take(&mut self.typed_slots[idx]);
        if clear_whole || prev_typed.chars().count() <= 1 {
            self.active_typed.clear();
        } else {
            let mut chars: Vec<char> = prev_typed.chars().collect();
            chars.pop();
            self.active_typed = chars.into_iter().collect();
        }
    }

    fn maybe_roll_line(
        &mut self,
        timestamp_ms: f64,
    ) -> Result<(bool, Option<LineRolloverPayload>), AksharError> {
        if self.grapheme_index < self.graphemes.len() {
            return Ok((false, None));
        }

        // Jungle lessons end once the lesson text finishes instead of rolling over infinitely.
        if self.mode == PracticeMode::Jungle {
            self.lines_completed += 1;
            return Ok((true, None));
        }

        if matches!(self.mode, PracticeMode::Adaptive | PracticeMode::WeakDrill) {
            let snap = self.metrics.snapshot(Some(timestamp_ms));
            let suffix = format!("l{}", self.line_index);
            self.persist_session_record(&snap, &suffix)?;
            self.storage.upsert_key_stats(&self.key_stats)?;
        }

        self.lines_completed += 1;
        self.line_index += 1;
        self.reachable_keys = reachable_unlocked_keys(self.layout, &self.key_stats, &self.corpus);
        self.load_next_line_words(None)?;

        Ok((
            true,
            Some(LineRolloverPayload {
                line_index: self.line_index,
                words: self.words.clone(),
                graphemes: self.graphemes.clone(),
            }),
        ))
    }

    fn persist_session_record(&mut self, snap: &LiveMetrics, suffix: &str) -> Result<(), AksharError> {
        let record = SessionRecord {
            id: format!("{}-{suffix}", self.session_id),
            layout: self.layout,
            mode: self.mode.as_str().to_string(),
            duration_ms: snap.elapsed_ms,
            gross_wpm: snap.gross_wpm,
            net_wpm: snap.net_wpm,
            kph: snap.kph,
            accuracy: snap.accuracy,
            consistency: snap.consistency,
            total_chars: snap.keystrokes,
            errors: snap.errors,
            created_at: Utc::now().to_rfc3339(),
        };
        self.storage.record_typing_session(&record)
    }

    fn load_next_line_words(&mut self, explicit: Option<Vec<String>>) -> Result<(), AksharError> {
        let mut r = rng();
        let words = match explicit {
            Some(w) => w,
            None => match self.mode {
                PracticeMode::Adaptive => generate_adaptive_line(
                    self.layout,
                    &self.key_stats,
                    &self.corpus,
                    self.words_per_line,
                    &mut r,
                )
                .or_else(|_| self.corpus.sample_words(self.layout, self.words_per_line, &mut r))?,
                PracticeMode::WeakDrill => generate_weak_drill_line(
                    self.layout,
                    &self.key_stats,
                    &self.drill_focus_keys,
                    &self.corpus,
                    self.words_per_line,
                    &mut r,
                )
                .or_else(|_| self.corpus.sample_words(self.layout, self.words_per_line, &mut r))?,
                PracticeMode::Exam => self.next_exam_chunk(),
                PracticeMode::Jungle | PracticeMode::MiniGame => self
                    .active_lesson
                    .as_ref()
                    .map(|l| l.words.clone())
                    .unwrap_or_default(),
            },
        };
        let joined = words.join(" ");
        let graphemes = split_graphemes(&joined);
        let len = graphemes.len();
        self.words = words;
        self.graphemes = graphemes;
        self.statuses = vec![GraphemeStatus::Pending; len];
        self.typed_slots = vec![String::new(); len];
        self.grapheme_index = 0;
        self.active_typed.clear();
        self.remington_ime.reset();
        Ok(())
    }

    fn next_exam_chunk(&mut self) -> Vec<String> {
        if self.exam_words.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(EXAM_WORDS_PER_CHUNK);
        let chunk_target = EXAM_WORDS_PER_CHUNK.min(self.exam_words.len());
        for _ in 0..chunk_target {
            if self.exam_cursor >= self.exam_words.len() {
                self.exam_cursor = 0;
            }
            out.push(self.exam_words[self.exam_cursor].clone());
            self.exam_cursor += 1;
        }
        out
    }

    fn recount_uncorrected_errors(&mut self) {
        let mut count = self
            .statuses
            .iter()
            .filter(|&&s| s == GraphemeStatus::Error)
            .count() as u32;
        if let Some(target) = self.graphemes.get(self.grapheme_index)
            && evaluate_grapheme(target, &self.active_typed) == GraphemeMatch::Mismatch
        {
            count += 1;
        }
        self.metrics.set_uncorrected_errors(count);
    }

    #[allow(clippy::too_many_arguments)]
    fn build_delta(
        &self,
        committed_updates: Vec<GraphemeUpdate>,
        sound_kind: KeySoundKind,
        line_completed: bool,
        next_line: Option<LineRolloverPayload>,
        newly_unlocked_key: Option<UnlockedKeyEvent>,
        updated_key_stats: Option<Vec<KeyStatRecord>>,
        now_ts_ms: Option<f64>,
    ) -> KeyEventDelta {
        let target = self
            .graphemes
            .get(self.grapheme_index)
            .map(String::as_str)
            .unwrap_or("");
        let eval = evaluate_grapheme(target, &self.active_typed);
        let pending_ime = if self.layout == LayoutKind::HiRemingtonGail {
            self.remington_ime.pending_preview().map(str::to_string)
        } else {
            None
        };
        let next_key_hint = hint_for_grapheme(
            self.layout,
            target,
            &self.active_typed,
            pending_ime.as_deref(),
        );

        KeyEventDelta {
            grapheme_index: self.grapheme_index as u32,
            active_typed_preview: self.active_typed.clone(),
            active_is_partial: matches!(eval, GraphemeMatch::Partial) || pending_ime.is_some(),
            active_has_error: matches!(eval, GraphemeMatch::Mismatch),
            committed_updates,
            pending_ime_preview: pending_ime,
            next_key_hint,
            metrics: self.metrics.snapshot(now_ts_ms),
            sound_kind,
            line_completed,
            next_line,
            newly_unlocked_key,
            updated_key_stats,
        }
    }
}

fn map_with_layout(
    layout: LayoutKind,
    key: PhysicalKey,
    event: &KeyInputEvent,
    current_buffer: &str,
    remington_ime: &mut RemingtonIme,
) -> (MappedKeyAction, String) {
    match layout {
        LayoutKind::EnQwerty => {
            let act = qwerty::map_key(key, event.shift, event.caps_lock);
            let next = apply_action_to_buffer(current_buffer, &act);
            (act, next)
        }
        LayoutKind::HiInscript => {
            let act = inscript::map_key(key, event.shift, event.alt_gr);
            let next = apply_action_to_buffer(current_buffer, &act);
            (act, next)
        }
        LayoutKind::HiRemingtonGail => {
            remington_ime.process_key(key, event.shift, event.alt_gr, current_buffer)
        }
    }
}

fn apply_action_to_buffer(current: &str, action: &MappedKeyAction) -> String {
    match action {
        MappedKeyAction::Insert(s) => {
            let mut out = current.to_string();
            out.push_str(s);
            out
        }
        MappedKeyAction::ReplaceTail {
            remove_chars,
            insert,
        } => {
            let mut chars: Vec<char> = current.chars().collect();
            let keep = chars.len().saturating_sub(*remove_chars);
            chars.truncate(keep);
            let mut out: String = chars.into_iter().collect();
            out.push_str(insert);
            out
        }
        MappedKeyAction::PendingPreview(_) | MappedKeyAction::NoOp => current.to_string(),
        MappedKeyAction::FlushSpace => " ".to_string(),
    }
}
