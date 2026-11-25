use std::rc::Rc;

use aksharflow_core::corpus::ExamPassage;
use aksharflow_core::curriculum::{
    CurriculumStage, JungleRoadmapSnapshot, LessonCompletionResult, LessonKind,
};
use aksharflow_core::grapheme::GraphemeStatus;
use aksharflow_core::keymap::{KeyInputEvent, KeycapDescriptor, LayoutKind, NextKeyHint};
use aksharflow_core::metrics::LiveMetrics;
use aksharflow_core::session::{
    ActiveLessonMeta, AnalyticsSnapshot, KeyEventDelta, PracticeMode, SessionEngine,
    SessionSnapshot, SessionSummary,
};
use aksharflow_core::storage::KeyStatRecord;
use craftui::prelude::*;

pub mod aot_ui {
    include!(concat!(env!("OUT_DIR"), "/aksharflow.rs"));
}

const DURATIONS_MINS: [i32; 5] = [1, 2, 5, 10, 15];
const SOUND_PROFILES: [&str; 5] = ["off", "blue", "brown", "red", "topre"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioTab {
    Jungle = 0,
    Adaptive = 1,
    Exam = 2,
    Analytics = 3,
}

#[derive(Debug, Clone)]
pub struct BalloonItem {
    pub id: u32,
    pub word: String,
    pub x_ratio: f32,
    pub y_px: f32,
    pub speed_px_per_sec: f32,
}

#[derive(Debug, Clone)]
pub struct PopParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
}

#[derive(Debug, Clone)]
pub struct BalloonWaveState {
    pub words: Vec<String>,
    pub balloons: Vec<BalloonItem>,
    pub particles: Vec<PopParticle>,
    pub next_word_idx: usize,
    pub next_id: u32,
    pub spawn_accum_ms: f32,
    pub paused: bool,
    pub score: u32,
    pub lives: u32,
    pub popped: u32,
    pub missed: u32,
    pub locked_word: Option<String>,
    pub typed_preview: String,
    pub input_error: bool,
    pub completed: bool,
    pub canvas_width: f32,
    pub canvas_height: f32,
}

impl Default for BalloonWaveState {
    fn default() -> Self {
        Self {
            words: vec!["akshar".to_string(), "flow".to_string()],
            balloons: Vec::new(),
            particles: Vec::new(),
            next_word_idx: 0,
            next_id: 1,
            spawn_accum_ms: 1500.0,
            paused: false,
            score: 0,
            lives: 5,
            popped: 0,
            missed: 0,
            locked_word: None,
            typed_preview: String::new(),
            input_error: false,
            completed: false,
            canvas_width: 760.0,
            canvas_height: 360.0,
        }
    }
}

pub type PersistJsonHook = Rc<dyn Fn(&str)>;
pub type ExportBackupHook = Rc<dyn Fn(&str) -> std::result::Result<String, String>>;
pub type ImportBackupHook = Rc<dyn Fn() -> Option<String>>;
pub type ThemeApplyHook = Rc<dyn Fn(bool)>;
pub type FocusNodeHook = Rc<dyn Fn(StudioTab, PracticeMode, bool)>;

pub struct AppCore {
    pub engine: SessionEngine,
    pub ui_tab: StudioTab,
    pub zen_mode: bool,
    pub theme_dark: bool,
    pub ui_lang_hi: bool,
    pub show_heatmap: bool,

    // Modifiers
    pub caps_lock: bool,
    pub shift_held: bool,
    pub alt_gr_held: bool,
    pub touch_shift: bool,
    pub touch_alt_gr: bool,

    // Current typing snapshot
    pub layout: LayoutKind,
    pub mode: PracticeMode,
    pub line_index: u32,
    pub graphemes: Vec<String>,
    pub statuses: Vec<GraphemeStatus>,
    pub grapheme_index: usize,
    pub active_typed_preview: String,
    pub active_is_partial: bool,
    pub active_has_error: bool,
    pub pending_ime_preview: Option<String>,
    pub next_key_hint: Option<NextKeyHint>,
    pub metrics: LiveMetrics,
    pub keycaps: Vec<KeycapDescriptor>,
    pub key_stats: Vec<KeyStatRecord>,
    pub active_lesson: Option<ActiveLessonMeta>,
    pub lesson_completion: Option<LessonCompletionResult>,

    // Jungle roadmap
    pub roadmap: Option<JungleRoadmapSnapshot>,
    pub selected_stage: usize,
    pub selected_unit: u32,
    pub visible_units: Vec<u32>,

    // Exam state
    pub exam_passages: Vec<ExamPassage>,
    pub selected_passage_idx: usize,
    pub exam_duration_mins: i32,
    pub exam_allow_backspace: bool,
    pub exam_remaining_secs: i32,
    pub exam_timer_running: bool,
    pub exam_summary: Option<SessionSummary>,

    // Analytics & mini-game
    pub analytics: Option<AnalyticsSnapshot>,
    pub balloon: BalloonWaveState,

    // Reactive signals read by CustomWidgets / root layout visibility
    pub revision: Signal<u64>,
    pub tab_sig: Signal<usize>,
    pub zen_sig: Signal<bool>,
    pub mode_sig: Signal<PracticeMode>,
    pub has_active_lesson_sig: Signal<bool>,
    pub tab_armed_for_restart: bool,

    // .cui panels
    pub top_bar: aot_ui::AksharTopBar,
    pub metrics_bar: aot_ui::AksharMetricsBar,
    pub unlock_strip: aot_ui::AksharUnlockStrip,
    pub jungle_panel: aot_ui::AksharJunglePanel,
    pub exam_bar: aot_ui::AksharExamBar,
    pub analytics_panel: aot_ui::AksharAnalyticsPanel,
    pub modals: aot_ui::AksharModalsOverlay,

    // Platform hooks
    pub persist_hook: Option<PersistJsonHook>,
    pub export_hook: Option<ExportBackupHook>,
    pub import_hook: Option<ImportBackupHook>,
    pub theme_hook: Option<ThemeApplyHook>,
    pub focus_hook: Option<FocusNodeHook>,
}

impl AppCore {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: SessionEngine,
        revision: Signal<u64>,
        tab_sig: Signal<usize>,
        zen_sig: Signal<bool>,
        mode_sig: Signal<PracticeMode>,
        has_active_lesson_sig: Signal<bool>,
        top_bar: aot_ui::AksharTopBar,
        metrics_bar: aot_ui::AksharMetricsBar,
        unlock_strip: aot_ui::AksharUnlockStrip,
        jungle_panel: aot_ui::AksharJunglePanel,
        exam_bar: aot_ui::AksharExamBar,
        analytics_panel: aot_ui::AksharAnalyticsPanel,
        modals: aot_ui::AksharModalsOverlay,
    ) -> Self {
        let mut core = Self {
            engine,
            ui_tab: StudioTab::Adaptive,
            zen_mode: false,
            theme_dark: true,
            ui_lang_hi: false,
            show_heatmap: true,
            caps_lock: false,
            shift_held: false,
            alt_gr_held: false,
            touch_shift: false,
            touch_alt_gr: false,
            layout: LayoutKind::EnQwerty,
            mode: PracticeMode::Adaptive,
            line_index: 0,
            graphemes: Vec::new(),
            statuses: Vec::new(),
            grapheme_index: 0,
            active_typed_preview: String::new(),
            active_is_partial: false,
            active_has_error: false,
            pending_ime_preview: None,
            next_key_hint: None,
            metrics: LiveMetrics::default(),
            keycaps: Vec::new(),
            key_stats: Vec::new(),
            active_lesson: None,
            lesson_completion: None,
            roadmap: None,
            selected_stage: 0,
            selected_unit: 1,
            visible_units: Vec::new(),
            exam_passages: Vec::new(),
            selected_passage_idx: 0,
            exam_duration_mins: 5,
            exam_allow_backspace: true,
            exam_remaining_secs: 300,
            exam_timer_running: false,
            exam_summary: None,
            analytics: None,
            balloon: BalloonWaveState::default(),
            revision,
            tab_sig,
            zen_sig,
            mode_sig,
            has_active_lesson_sig,
            tab_armed_for_restart: false,
            top_bar,
            metrics_bar,
            unlock_strip,
            jungle_panel,
            exam_bar,
            analytics_panel,
            modals,
            persist_hook: None,
            export_hook: None,
            import_hook: None,
            theme_hook: None,
            focus_hook: None,
        };
        core.bootstrap();
        core
    }

    pub fn bump(&self) {
        self.revision.update(|n| *n = n.wrapping_add(1));
    }

    fn persist_snapshot(&self) {
        if let Some(ref hook) = self.persist_hook
            && let Ok(json) = self.engine.export_backup_json()
        {
            hook(&json);
        }
    }

    fn sync_chrome_labels(&self) {
        let hi = self.ui_lang_hi;
        self.top_bar
            .app_title
            .set(if hi { "अक्षर-फ्लो".into() } else { "AksharFlow".into() });
        self.top_bar.app_sub.set(if hi {
            "700 पाठ व एडैप्टिव टाइपिंग स्टूडियो".into()
        } else {
            "Offline Adaptive & 700-Lesson Studio".into()
        });
        self.top_bar
            .label_jungle
            .set(if hi { "जंगल यात्रा".into() } else { "Jungle Journey".into() });
        self.top_bar
            .label_adaptive
            .set(if hi { "स्मार्ट एडैप्टिव".into() } else { "Smart Adaptive".into() });
        self.top_bar
            .label_exam
            .set(if hi { "परीक्षा मोड".into() } else { "Exam Mode".into() });
        self.top_bar
            .label_analytics
            .set(if hi { "विश्लेषण".into() } else { "Analytics".into() });

        self.top_bar.zen_mode.set(self.zen_mode);
        self.top_bar
            .is_analytics
            .set(self.ui_tab == StudioTab::Analytics);
        self.top_bar
            .is_tab_jungle
            .set(self.ui_tab == StudioTab::Jungle);
        self.top_bar
            .is_tab_adaptive
            .set(self.ui_tab == StudioTab::Adaptive);
        self.top_bar
            .is_tab_exam
            .set(self.ui_tab == StudioTab::Exam);
        self.top_bar
            .is_tab_analytics
            .set(self.ui_tab == StudioTab::Analytics);

        self.top_bar
            .is_qwerty
            .set(self.layout == LayoutKind::EnQwerty);
        self.top_bar
            .is_inscript
            .set(self.layout == LayoutKind::HiInscript);
        self.top_bar
            .is_remington
            .set(self.layout == LayoutKind::HiRemingtonGail);

        self.top_bar
            .lang_label
            .set(if hi { "हिं".into() } else { "EN".into() });
        self.top_bar
            .theme_label
            .set(if self.theme_dark { "Light".into() } else { "Dark".into() });
        self.top_bar
            .zen_label
            .set(if hi { "फ़ोकस".into() } else { "Zen".into() });
        self.top_bar
            .exit_zen_label
            .set(if hi { "फ़ोकस बंद".into() } else { "Exit Zen".into() });
        self.top_bar
            .settings_label
            .set(if hi { "सेटिंग्स".into() } else { "Settings".into() });

        let snd = self.engine.settings().sound_profile.as_str();
        self.top_bar.sound_label.set(format!("Audio: {snd}"));
    }

    pub fn bootstrap(&mut self) {
        if let Ok(boot) = self.engine.bootstrap(Some(18)) {
            let s = &boot.settings;
            self.ui_lang_hi = s.ui_lang == "hi";
            self.theme_dark = s.theme != "light";
            self.zen_mode = s.zen_mode;
            self.key_stats = boot.key_stats;
            self.modals.target_wpm.set(s.target_wpm as f32);
            self.modals.target_accuracy.set(s.target_accuracy as f32);
            self.modals.unlock_threshold.set(s.unlock_threshold as f32);
            self.apply_session_snapshot(boot.session);
        }
        if let Ok(passages) = self.engine.list_exam_passages() {
            self.exam_passages = passages;
            self.pick_default_passage_for_layout();
        }
        self.zen_sig.set(self.zen_mode);
        self.sync_chrome_labels();
        self.sync_unlock_strip();
        self.sync_metrics_bar();
        if let Some(ref hook) = self.theme_hook {
            hook(self.theme_dark);
        }
        self.bump();
    }

    fn apply_session_snapshot(&mut self, snap: SessionSnapshot) {
        self.layout = snap.layout;
        self.mode = snap.mode;
        self.line_index = snap.line_index;
        self.graphemes = snap.graphemes;
        self.statuses = vec![GraphemeStatus::Pending; self.graphemes.len()];
        self.grapheme_index = snap.delta.grapheme_index as usize;
        self.active_typed_preview = snap.delta.active_typed_preview;
        self.active_is_partial = snap.delta.active_is_partial;
        self.active_has_error = snap.delta.active_has_error;
        self.pending_ime_preview = snap.delta.pending_ime_preview;
        self.next_key_hint = snap.delta.next_key_hint;
        self.metrics = snap.delta.metrics;
        self.keycaps = snap.keycaps;
        self.active_lesson = snap.lesson_meta;
        self.mode_sig.set(self.mode);
        self.has_active_lesson_sig.set(self.active_lesson.is_some());

        if self.mode == PracticeMode::MiniGame {
            self.balloon = BalloonWaveState {
                words: if snap.words.is_empty() {
                    vec!["akshar".to_string(), "flow".to_string()]
                } else {
                    snap.words
                },
                canvas_width: self.balloon.canvas_width,
                canvas_height: self.balloon.canvas_height,
                ..Default::default()
            };
        }
    }

    fn apply_delta(&mut self, delta: KeyEventDelta) {
        let should_start_exam_timer = self.mode == PracticeMode::Exam
            && !self.exam_timer_running
            && self.exam_remaining_secs > 0
            && delta.metrics.keystrokes > 0
            && self.exam_summary.is_none();

        let finished_jungle = self.mode == PracticeMode::Jungle
            && delta.line_completed
            && delta.next_line.is_none()
            && self.active_lesson.is_some()
            && self.lesson_completion.is_none();

        if delta.line_completed
            && let Some(next_line) = delta.next_line
        {
            self.line_index = next_line.line_index;
            self.graphemes = next_line.graphemes;
            self.statuses = vec![GraphemeStatus::Pending; self.graphemes.len()];
        } else {
            for patch in delta.committed_updates {
                let idx = patch.index as usize;
                if idx < self.statuses.len() {
                    self.statuses[idx] = patch.status;
                }
            }
        }

        self.grapheme_index = delta.grapheme_index as usize;
        self.active_typed_preview = delta.active_typed_preview;
        self.active_is_partial = delta.active_is_partial;
        self.active_has_error = delta.active_has_error;
        self.pending_ime_preview = delta.pending_ime_preview;
        self.next_key_hint = delta.next_key_hint;
        self.metrics = delta.metrics;

        if let Some(stats) = delta.updated_key_stats {
            self.key_stats = stats;
            self.sync_unlock_strip();
        }
        if let Some(unlocked) = delta.newly_unlocked_key {
            let msg = format!(
                "New Key Unlocked: {} ({}/{})",
                unlocked.key_id, unlocked.unlock_order, unlocked.total_keys
            );
            self.modals.unlock_toast_text.set(msg);
            self.modals.show_unlock_toast.set(true);
            self.persist_snapshot();
        } else if delta.line_completed {
            self.persist_snapshot();
        }
        if should_start_exam_timer {
            self.exam_timer_running = true;
        }

        self.sync_metrics_bar();
        if finished_jungle {
            self.complete_active_jungle_lesson(None);
        }
        self.bump();
    }

    fn sync_metrics_bar(&self) {
        self.metrics_bar
            .net_wpm
            .set(format!("{}", self.metrics.net_wpm.round() as i32));
        self.metrics_bar
            .gross_wpm
            .set(format!("{}", self.metrics.gross_wpm.round() as i32));
        self.metrics_bar
            .accuracy
            .set(format!("{:.1}%", self.metrics.accuracy));
        self.metrics_bar
            .consistency
            .set(format!("{}%", self.metrics.consistency.round() as i32));
        self.metrics_bar
            .line_label
            .set(format!("#{}", self.line_index + 1));
        let secs = self.metrics.elapsed_ms / 1000;
        self.metrics_bar
            .time_label
            .set(format!("{:02}:{:02}", secs / 60, secs % 60));

        self.metrics_bar.show_caps_warning.set(self.caps_lock);
        let is_hindi = self.layout != LayoutKind::EnQwerty;
        self.metrics_bar.caps_text.set(if is_hindi {
            "CapsLock ON (Ignored in Hindi)".into()
        } else {
            "CapsLock ON".into()
        });

        self.metrics_bar
            .show_pending_ime
            .set(self.pending_ime_preview.is_some());
        if let Some(ref pend) = self.pending_ime_preview {
            self.metrics_bar
                .pending_ime_text
                .set(format!("Pending: {pend}"));
        }

        let is_jungle_typing = self.mode == PracticeMode::Jungle && self.active_lesson.is_some();
        self.metrics_bar.is_jungle_lesson.set(is_jungle_typing);
        if let Some(ref meta) = self.active_lesson {
            let title = if self.ui_lang_hi {
                &meta.title_hi
            } else {
                &meta.title_en
            };
            self.metrics_bar.lesson_header.set(format!(
                "{} | {} | Target {} WPM / {}%",
                meta.id.to_uppercase(),
                title,
                meta.target_wpm.round() as i32,
                meta.min_accuracy.round() as i32
            ));
        }

        self.metrics_bar
            .show_restart
            .set(self.mode != PracticeMode::Exam);
        self.metrics_bar.restart_label.set(match self.mode {
            PracticeMode::Jungle => "Restart Lesson".into(),
            PracticeMode::WeakDrill => "Exit Drill".into(),
            _ => "New Line".into(),
        });
    }

    fn sync_unlock_strip(&self) {
        let threshold = self.engine.settings().unlock_threshold;
        let unlocked_stats: Vec<&KeyStatRecord> =
            self.key_stats.iter().filter(|k| k.unlocked).collect();
        let mut sorted = unlocked_stats.clone();
        sorted.sort_by(|a, b| {
            let a_unseen = (a.hits + a.misses == 0) as u8;
            let b_unseen = (b.hits + b.misses == 0) as u8;
            b_unseen.cmp(&a_unseen).then_with(|| {
                b.weakness_weight
                    .partial_cmp(&a.weakness_weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
        let top3: Vec<&str> = sorted.iter().take(3).map(|k| k.key_id.as_str()).collect();

        self.unlock_strip.summary_text.set(format!(
            "Unlocked Keys: {}/{}",
            unlocked_stats.len(),
            self.key_stats.len()
        ));
        self.unlock_strip
            .threshold_text
            .set(format!("Unlock goal: {}% confidence", threshold.round() as i32));

        let chips: Vec<aot_ui::KeyChipItem> = self
            .key_stats
            .iter()
            .map(|k| {
                let total = k.hits + k.misses;
                let mastered = k.unlocked && total >= 12 && k.confidence_score >= threshold;
                let weak_focus = k.unlocked && !mastered && top3.contains(&k.key_id.as_str());
                aot_ui::KeyChipItem {
                    label: k.key_id.clone(),
                    sub_label: format!("{}%", k.confidence_score.round() as i32),
                    unlocked: k.unlocked,
                    mastered,
                    weak_focus,
                }
            })
            .collect();
        self.unlock_strip.chips.set(chips);
    }

    pub fn select_tab(&mut self, tab_idx: usize) {
        let next_tab = match tab_idx {
            0 => StudioTab::Jungle,
            1 => StudioTab::Adaptive,
            2 => StudioTab::Exam,
            _ => StudioTab::Analytics,
        };
        let prev_tab = self.ui_tab;
        self.ui_tab = next_tab;
        self.tab_sig.set(tab_idx);
        self.modals.lesson_modal_open.set(false);
        self.modals.exam_modal_open.set(false);
        self.lesson_completion = None;
        self.exam_summary = None;

        match next_tab {
            StudioTab::Jungle => {
                self.exam_timer_running = false;
                if prev_tab == StudioTab::Jungle || self.active_lesson.is_none() {
                    self.active_lesson = None;
                    self.has_active_lesson_sig.set(false);
                    if self.mode == PracticeMode::MiniGame || self.mode == PracticeMode::Jungle {
                        self.mode = PracticeMode::Adaptive;
                        self.mode_sig.set(self.mode);
                    }
                    self.reload_jungle_roadmap();
                }
            }
            StudioTab::Adaptive => {
                self.exam_timer_running = false;
                self.active_lesson = None;
                self.has_active_lesson_sig.set(false);
                if !matches!(self.mode, PracticeMode::Adaptive | PracticeMode::WeakDrill)
                    && let Ok(snap) = self.engine.start_session(self.layout, Some(18))
                {
                    self.apply_session_snapshot(snap);
                    self.key_stats = self.engine.key_stats().to_vec();
                }
            }
            StudioTab::Exam => {
                self.active_lesson = None;
                self.has_active_lesson_sig.set(false);
                self.start_exam_with_selected();
            }
            StudioTab::Analytics => {
                self.exam_timer_running = false;
                self.reload_analytics();
            }
        }

        self.sync_chrome_labels();
        self.sync_metrics_bar();
        self.sync_unlock_strip();
        self.trigger_focus();
        self.bump();
    }

    pub fn select_layout(&mut self, layout_idx: usize) {
        let next = match layout_idx {
            1 => LayoutKind::HiInscript,
            2 => LayoutKind::HiRemingtonGail,
            _ => LayoutKind::EnQwerty,
        };
        self.layout = next;
        self.touch_shift = false;
        self.touch_alt_gr = false;
        self.active_lesson = None;
        self.has_active_lesson_sig.set(false);
        self.lesson_completion = None;
        self.exam_timer_running = false;
        self.exam_summary = None;
        self.pick_default_passage_for_layout();

        match self.ui_tab {
            StudioTab::Exam => self.start_exam_with_selected(),
            StudioTab::Jungle => {
                let _ = self.engine.start_session(next, Some(18));
                self.key_stats = self.engine.key_stats().to_vec();
                self.reload_jungle_roadmap();
            }
            StudioTab::Analytics => {
                if let Ok(snap) = self.engine.start_session(next, Some(18)) {
                    self.apply_session_snapshot(snap);
                    self.key_stats = self.engine.key_stats().to_vec();
                }
                self.reload_analytics();
            }
            StudioTab::Adaptive => {
                if let Ok(snap) = self.engine.start_session(next, Some(18)) {
                    self.apply_session_snapshot(snap);
                    self.key_stats = self.engine.key_stats().to_vec();
                }
            }
        }
        self.sync_chrome_labels();
        self.sync_metrics_bar();
        self.sync_unlock_strip();
        self.persist_snapshot();
        self.trigger_focus();
        self.bump();
    }

    pub fn toggle_lang(&mut self) {
        self.ui_lang_hi = !self.ui_lang_hi;
        let mut s = self.engine.settings().clone();
        s.ui_lang = if self.ui_lang_hi { "hi".into() } else { "en".into() };
        let _ = self.engine.update_settings(s);
        self.sync_chrome_labels();
        self.sync_metrics_bar();
        if self.ui_tab == StudioTab::Jungle && self.active_lesson.is_none() {
            self.sync_jungle_panel();
        }
        self.persist_snapshot();
        self.bump();
    }

    pub fn toggle_theme(&mut self) {
        self.theme_dark = !self.theme_dark;
        let mut s = self.engine.settings().clone();
        s.theme = if self.theme_dark {
            "dark".into()
        } else {
            "light".into()
        };
        let _ = self.engine.update_settings(s);
        self.sync_chrome_labels();
        if let Some(ref hook) = self.theme_hook {
            hook(self.theme_dark);
        }
        self.persist_snapshot();
        self.bump();
    }

    pub fn toggle_zen(&mut self) {
        self.zen_mode = !self.zen_mode;
        self.zen_sig.set(self.zen_mode);
        let mut s = self.engine.settings().clone();
        s.zen_mode = self.zen_mode;
        let _ = self.engine.update_settings(s);
        self.sync_chrome_labels();
        self.persist_snapshot();
        self.trigger_focus();
        self.bump();
    }

    pub fn cycle_sound(&mut self) {
        let mut s = self.engine.settings().clone();
        let idx = SOUND_PROFILES
            .iter()
            .position(|&p| p == s.sound_profile)
            .unwrap_or(2);
        s.sound_profile = SOUND_PROFILES[(idx + 1) % SOUND_PROFILES.len()].to_string();
        let _ = self.engine.update_settings(s);
        self.sync_chrome_labels();
        self.persist_snapshot();
    }

    pub fn trigger_focus(&self) {
        if let Some(ref hook) = self.focus_hook {
            hook(self.ui_tab, self.mode, self.active_lesson.is_some());
        }
    }

    pub fn restart_or_exit(&mut self) {
        if let Some(ref meta) = self.active_lesson {
            let id = meta.id.clone();
            self.start_jungle_lesson(&id);
        } else if let Ok(snap) = self.engine.start_session(self.layout, Some(18)) {
            self.apply_session_snapshot(snap);
            self.key_stats = self.engine.key_stats().to_vec();
            self.sync_metrics_bar();
            self.sync_unlock_strip();
            self.trigger_focus();
            self.bump();
        }
    }

    pub fn back_to_jungle_map(&mut self) {
        self.ui_tab = StudioTab::Jungle;
        self.tab_sig.set(StudioTab::Jungle as usize);
        self.active_lesson = None;
        self.has_active_lesson_sig.set(false);
        self.lesson_completion = None;
        self.modals.lesson_modal_open.set(false);
        self.mode = PracticeMode::Adaptive;
        self.mode_sig.set(self.mode);
        self.reload_jungle_roadmap();
        self.sync_chrome_labels();
        self.bump();
    }

    pub fn reload_jungle_roadmap(&mut self) {
        if let Ok(rm) = self.engine.get_jungle_roadmap(self.layout) {
            if let Some(next_unit) = rm
                .units
                .iter()
                .find(|u| u.lessons.iter().any(|l| l.id == rm.next_lesson_id))
            {
                self.selected_unit = next_unit.unit_number;
            }
            self.roadmap = Some(rm);
            self.sync_jungle_panel();
        }
    }

    pub fn select_stage(&mut self, stage_idx: usize) {
        self.selected_stage = stage_idx;
        if let Some(ref rm) = self.roadmap {
            let matching = rm.units.iter().find(|u| stage_matches(stage_idx, u.stage));
            if let Some(first) = matching {
                self.selected_unit = first.unit_number;
            }
        }
        self.sync_jungle_panel();
    }

    pub fn select_visible_unit(&mut self, idx: usize) {
        if let Some(&unit_num) = self.visible_units.get(idx) {
            self.selected_unit = unit_num;
            self.sync_jungle_panel();
        }
    }

    fn sync_jungle_panel(&mut self) {
        let Some(ref rm) = self.roadmap else {
            return;
        };
        let hi = self.ui_lang_hi;
        self.jungle_panel.title.set(if hi {
            "जंगल यात्रा".into()
        } else {
            "Jungle Journey".into()
        });
        self.jungle_panel.subtitle.set(if hi {
            "28 इकाइयों में 700 क्रमिक टाइपिंग पाठ".into()
        } else {
            "700 structured lessons across 28 progressive units".into()
        });
        self.jungle_panel.progress_summary.set(format!(
            "{} / {} stars  |  {} / {} lessons",
            rm.total_earned_stars, rm.total_max_stars, rm.completed_lessons, rm.total_lessons
        ));
        self.jungle_panel.continue_label.set(format!(
            "{} ({})",
            if hi { "जारी रखें" } else { "Continue Next" },
            rm.next_lesson_id.to_uppercase()
        ));

        self.visible_units.clear();
        let mut unit_items = Vec::new();
        for u in &rm.units {
            if !stage_matches(self.selected_stage, u.stage) {
                continue;
            }
            self.visible_units.push(u.unit_number);
            let title = if hi { &u.title_hi } else { &u.title_en };
            unit_items.push(aot_ui::UnitItem {
                code: format!("U{:02}", u.unit_number),
                title: title.clone(),
                meta: format!(
                    "{}/{} | Keys: {}",
                    u.completed_lessons,
                    u.lessons.len(),
                    u.focus_keys.join(" ")
                ),
                stars_text: format!("{}/{}", u.earned_stars, u.max_stars),
                selected: u.unit_number == self.selected_unit,
                unlocked: u.unlocked,
            });
        }
        self.jungle_panel.units.set(unit_items);

        let active_unit = rm
            .units
            .iter()
            .find(|u| u.unit_number == self.selected_unit)
            .or_else(|| {
                rm.units
                    .iter()
                    .find(|u| stage_matches(self.selected_stage, u.stage))
            });

        if let Some(u) = active_unit {
            let title = if hi { &u.title_hi } else { &u.title_en };
            self.jungle_panel
                .active_unit_title
                .set(format!("Unit {}: {}", u.unit_number, title));
            self.jungle_panel.active_unit_meta.set(format!(
                "Target: {} WPM / {}% | Focus: {}",
                u.target_wpm.round() as i32,
                u.min_accuracy.round() as i32,
                u.focus_keys.join(" ")
            ));

            // Arrange 25 lessons in alternating left-right serpentine rows of 5
            let mut ordered = Vec::with_capacity(u.lessons.len());
            for (row_idx, chunk) in u.lessons.chunks(5).enumerate() {
                if row_idx % 2 == 1 {
                    for l in chunk.iter().rev() {
                        ordered.push(l);
                    }
                } else {
                    ordered.extend(chunk.iter());
                }
            }

            let lesson_items: Vec<aot_ui::LessonItem> = ordered
                .into_iter()
                .map(|l| {
                    let kind_label = match (l.kind, hi) {
                        (LessonKind::KeyIntro, false) => "Key Intro",
                        (LessonKind::Drill, false) => "Rhythm Drill",
                        (LessonKind::Words, false) => "Word Builder",
                        (LessonKind::Sentences, false) => "Sentences",
                        (LessonKind::ReviewChallenge, false) => "Unit Boss",
                        (LessonKind::MiniGame, false) => "Balloon Game",
                        (LessonKind::KeyIntro, true) => "कुंजी परिचय",
                        (LessonKind::Drill, true) => "लय अभ्यास",
                        (LessonKind::Words, true) => "शब्द अभ्यास",
                        (LessonKind::Sentences, true) => "वाक्य अभ्यास",
                        (LessonKind::ReviewChallenge, true) => "इकाई परीक्षा",
                        (LessonKind::MiniGame, true) => "बैलून गेम",
                    };
                    let filled = "★".repeat(l.stars as usize);
                    let empty = "☆".repeat(5usize.saturating_sub(l.stars as usize));
                    aot_ui::LessonItem {
                        id: l.id.clone(),
                        num_label: if l.unlocked {
                            format!("#{}", l.global_number)
                        } else {
                            format!("#{} Lock", l.global_number)
                        },
                        kind_label: kind_label.to_string(),
                        stars_text: format!("{filled}{empty}"),
                        wpm_text: if l.best_wpm > 0.0 {
                            format!("{}w", l.best_wpm.round() as i32)
                        } else {
                            String::new()
                        },
                        unlocked: l.unlocked,
                        is_next: l.id == rm.next_lesson_id,
                        passed: l.stars >= 3,
                    }
                })
                .collect();
            self.jungle_panel.lessons.set(lesson_items);
        }
    }

    pub fn start_jungle_lesson(&mut self, lesson_id: &str) {
        if let Some(ref rm) = self.roadmap {
            let is_unlocked = rm
                .units
                .iter()
                .flat_map(|u| u.lessons.iter())
                .find(|l| l.id == lesson_id)
                .is_none_or(|l| l.unlocked);
            if !is_unlocked {
                return;
            }
        }
        if let Ok(snap) = self.engine.start_jungle_lesson(self.layout, lesson_id) {
            self.ui_tab = StudioTab::Jungle;
            self.tab_sig.set(StudioTab::Jungle as usize);
            self.lesson_completion = None;
            self.modals.lesson_modal_open.set(false);
            self.apply_session_snapshot(snap);
            self.sync_chrome_labels();
            self.sync_metrics_bar();
            self.trigger_focus();
            self.bump();
        }
    }

    pub fn complete_active_jungle_lesson(
        &mut self,
        override_stats: Option<(f64, f64, u32, u32)>,
    ) {
        let res = match override_stats {
            Some((wpm, acc, err, keys)) => self
                .engine
                .complete_jungle_lesson(Some(wpm), Some(acc), Some(err), Some(keys)),
            None => self.engine.complete_jungle_lesson(None, None, None, None),
        };
        if let Ok(done) = res {
            let filled = "★".repeat(done.stars_earned as usize);
            let empty = "☆".repeat(5usize.saturating_sub(done.stars_earned as usize));
            self.modals.lesson_stars.set(format!("{filled}{empty}"));
            self.modals
                .lesson_wpm
                .set(format!("{} / {}", done.net_wpm.round() as i32, done.target_wpm.round() as i32));
            self.modals.lesson_accuracy.set(format!(
                "{:.1}% / {}%",
                done.accuracy,
                done.min_accuracy.round() as i32
            ));
            let title = self
                .active_lesson
                .as_ref()
                .map(|m| {
                    if self.ui_lang_hi {
                        m.title_hi.clone()
                    } else {
                        m.title_en.clone()
                    }
                })
                .unwrap_or_else(|| done.lesson_id.clone());
            self.modals.lesson_title.set(title);
            self.modals.lesson_status.set(if done.passed {
                "Next lesson unlocked!".to_string()
            } else {
                "Reach 3 stars (75%+ accuracy and steady speed) to unlock next node.".to_string()
            });
            self.modals.can_next_lesson.set(
                done.next_lesson_id.is_some() && (done.next_lesson_unlocked || done.best_stars >= 3),
            );
            self.modals.lesson_modal_open.set(true);
            self.lesson_completion = Some(done);
            self.reload_jungle_roadmap();
            self.persist_snapshot();
            self.bump();
        }
    }

    pub fn start_next_jungle_lesson(&mut self) {
        let next_id = self
            .lesson_completion
            .as_ref()
            .and_then(|c| c.next_lesson_id.clone())
            .or_else(|| self.roadmap.as_ref().map(|r| r.next_lesson_id.clone()));
        if let Some(id) = next_id {
            self.start_jungle_lesson(&id);
        }
    }

    pub fn reload_analytics(&mut self) {
        if let Ok(snap) = self.engine.analytics(self.layout) {
            self.analytics_panel.jungle_summary.set(format!(
                "{} / {} Stars  |  {} / {} Lessons",
                snap.jungle_earned_stars,
                snap.jungle_max_stars,
                snap.jungle_completed_lessons,
                snap.jungle_total_lessons
            ));
            self.analytics_panel
                .avg_wpm
                .set(format!("{}", snap.avg_net_wpm.round() as i32));
            self.analytics_panel
                .best_wpm
                .set(format!("{}", snap.best_net_wpm.round() as i32));
            self.analytics_panel
                .avg_acc
                .set(format!("{:.1}%", snap.avg_accuracy));
            self.analytics_panel
                .total_sessions
                .set(format!("{}", snap.total_sessions));
            self.analytics_panel
                .unlocked_ratio
                .set(format!("{}/{}", snap.unlocked_count, snap.total_keys));

            let weak = top_weak_keys(&snap.key_stats, 5);
            let weak_rows: Vec<aot_ui::WeakKeyRow> = weak
                .iter()
                .map(|w| {
                    let total = w.hits + w.misses;
                    let acc = if total > 0 {
                        (w.hits as f64 / total as f64) * 100.0
                    } else {
                        100.0
                    };
                    aot_ui::WeakKeyRow {
                        key_label: w.key_id.clone(),
                        metrics_text: format!("{acc:.0}% | {}ms", w.ema_latency_ms.round() as i32),
                        confidence_text: format!(
                            "{}% ({} hits)",
                            w.confidence_score.round() as i32,
                            total
                        ),
                    }
                })
                .collect();
            self.analytics_panel.weak_keys.set(weak_rows);

            let sess_rows: Vec<aot_ui::SessionHistoryRow> = snap
                .recent_sessions
                .iter()
                .take(25)
                .map(|s| aot_ui::SessionHistoryRow {
                    mode: s.mode.replace('_', " "),
                    wpm: format!("{}", s.net_wpm.round() as i32),
                    accuracy: format!("{:.1}", s.accuracy),
                    kph: format!("{}", s.kph.round() as i32),
                    errors: format!("{}", s.errors),
                })
                .collect();
            self.analytics_panel.recent_sessions.set(sess_rows);
            self.analytics = Some(snap);
        }
    }

    pub fn start_weak_key_drill(&mut self) {
        let keys: Vec<String> = top_weak_keys(&self.key_stats, 5)
            .into_iter()
            .map(|k| k.key_id)
            .collect();
        if keys.is_empty() {
            return;
        }
        if let Ok(boot) = self
            .engine
            .start_weak_key_drill(self.layout, keys, Some(18))
        {
            self.ui_tab = StudioTab::Adaptive;
            self.tab_sig.set(StudioTab::Adaptive as usize);
            self.active_lesson = None;
            self.has_active_lesson_sig.set(false);
            self.apply_session_snapshot(boot.session);
            self.sync_chrome_labels();
            self.sync_metrics_bar();
            self.sync_unlock_strip();
            self.trigger_focus();
            self.bump();
        }
    }

    fn pick_default_passage_for_layout(&mut self) {
        let target_lang = if self.layout == LayoutKind::EnQwerty {
            "en"
        } else {
            "hi"
        };
        if let Some(pos) = self
            .exam_passages
            .iter()
            .position(|p| p.language == target_lang)
        {
            self.selected_passage_idx = pos;
        } else {
            self.selected_passage_idx = 0;
        }
        self.sync_exam_bar_labels();
    }

    fn sync_exam_bar_labels(&self) {
        let title = self
            .exam_passages
            .get(self.selected_passage_idx)
            .map(|p| p.title.clone())
            .unwrap_or_else(|| "No passage".to_string());
        self.exam_bar.passage_label.set(title);
        let mins = self.exam_remaining_secs / 60;
        let secs = self.exam_remaining_secs % 60;
        self.exam_bar
            .remaining_label
            .set(format!("{mins:02}:{secs:02}"));
    }

    pub fn set_exam_duration_idx(&mut self, idx: usize) {
        if let Some(&mins) = DURATIONS_MINS.get(idx) {
            self.exam_duration_mins = mins;
            if self.ui_tab == StudioTab::Exam {
                self.start_exam_with_selected();
            }
        }
    }

    pub fn step_exam_passage(&mut self, delta: isize) {
        if self.exam_passages.is_empty() {
            return;
        }
        let len = self.exam_passages.len() as isize;
        self.selected_passage_idx =
            ((self.selected_passage_idx as isize + delta).rem_euclid(len)) as usize;
        if self.ui_tab == StudioTab::Exam {
            self.start_exam_with_selected();
        } else {
            self.sync_exam_bar_labels();
        }
    }

    pub fn set_exam_strict(&mut self, strict_no_backspace: bool) {
        self.exam_allow_backspace = !strict_no_backspace;
        if self.ui_tab == StudioTab::Exam {
            self.start_exam_with_selected();
        }
    }

    pub fn start_exam_with_selected(&mut self) {
        let content = self
            .exam_passages
            .get(self.selected_passage_idx)
            .map(|p| p.content.clone())
            .unwrap_or_default();
        if content.trim().is_empty() {
            return;
        }
        let duration_secs = (self.exam_duration_mins * 60) as u32;
        if let Ok(snap) = self.engine.start_exam_session(
            self.layout,
            content,
            duration_secs,
            self.exam_allow_backspace,
        ) {
            self.exam_remaining_secs = duration_secs as i32;
            self.exam_timer_running = false;
            self.exam_summary = None;
            self.modals.exam_modal_open.set(false);
            self.apply_session_snapshot(snap);
            self.sync_exam_bar_labels();
            self.sync_metrics_bar();
            self.trigger_focus();
            self.bump();
        }
    }

    pub fn save_custom_passage(&mut self) {
        let title = self.exam_bar.custom_title.get_untracked();
        let body = self.exam_bar.custom_body.get_untracked();
        if title.trim().is_empty() || body.trim().is_empty() {
            return;
        }
        let lang = if self.layout == LayoutKind::EnQwerty {
            "en"
        } else {
            "hi"
        };
        if self
            .engine
            .save_custom_passage(title, lang.into(), body)
            .is_ok()
            && let Ok(all) = self.engine.list_exam_passages()
        {
            self.exam_passages = all;
            self.selected_passage_idx = self.exam_passages.len().saturating_sub(1);
            self.exam_bar.custom_title.set(String::new());
            self.exam_bar.custom_body.set(String::new());
            self.exam_bar.show_custom_form.set(false);
            self.start_exam_with_selected();
            self.persist_snapshot();
        }
    }

    pub fn finish_exam(&mut self) {
        self.exam_timer_running = false;
        if let Ok(summary) = self.engine.finish_session() {
            let m = &summary.metrics;
            self.modals.exam_metrics_line1.set(format!(
                "Net {} WPM  |  Gross {} WPM  |  KPH {}",
                m.net_wpm.round() as i32,
                m.gross_wpm.round() as i32,
                m.kph.round() as i32
            ));
            self.modals.exam_metrics_line2.set(format!(
                "Accuracy {:.1}%  |  Consistency {}%  |  Errors {}  |  Chunks {}",
                m.accuracy,
                m.consistency.round() as i32,
                m.errors,
                summary.lines_completed
            ));
            self.exam_summary = Some(summary);
            self.modals.exam_modal_open.set(true);
            self.persist_snapshot();
            self.bump();
        }
    }

    pub fn tick_exam_1s(&mut self) {
        if !self.exam_timer_running || self.mode != PracticeMode::Exam {
            return;
        }
        if self.exam_remaining_secs <= 1 {
            self.exam_remaining_secs = 0;
            self.sync_exam_bar_labels();
            self.finish_exam();
        } else {
            self.exam_remaining_secs -= 1;
            self.sync_exam_bar_labels();
        }
    }

    pub fn tick_game_frame(&mut self, dt_ms: f32) {
        if self.mode != PracticeMode::MiniGame
            || self.balloon.paused
            || self.balloon.completed
            || self.lesson_completion.is_some()
        {
            return;
        }
        let dt_sec = dt_ms / 1000.0;
        let height = self.balloon.canvas_height.max(240.0);

        self.balloon.spawn_accum_ms += dt_ms;
        if self.balloon.spawn_accum_ms >= 2100.0
            && self.balloon.balloons.len() < 4
            && self.balloon.next_word_idx < self.balloon.words.len()
        {
            self.balloon.spawn_accum_ms = 0.0;
            let w = self.balloon.words[self.balloon.next_word_idx].clone();
            self.balloon.next_word_idx += 1;
            let id = self.balloon.next_id;
            self.balloon.next_id += 1;
            let lane = ((id % 5) as f32) * 0.16 + 0.18;
            let speed = 34.0 + (self.balloon.popped as f32 * 1.1).min(18.0);
            self.balloon.balloons.push(BalloonItem {
                id,
                word: w,
                x_ratio: lane.clamp(0.16, 0.84),
                y_px: 44.0,
                speed_px_per_sec: speed,
            });
        }

        let mut survivors = Vec::with_capacity(self.balloon.balloons.len());
        let mut lost = 0u32;
        for mut b in self.balloon.balloons.drain(..) {
            b.y_px += b.speed_px_per_sec * dt_sec;
            if b.y_px >= height - 36.0 {
                lost += 1;
                if self.balloon.locked_word.as_deref() == Some(b.word.as_str()) {
                    self.balloon.locked_word = None;
                    self.balloon.typed_preview.clear();
                }
            } else {
                survivors.push(b);
            }
        }
        self.balloon.balloons = survivors;

        if lost > 0 {
            self.balloon.missed += lost;
            self.balloon.lives = self.balloon.lives.saturating_sub(lost);
        }

        self.balloon.particles.retain_mut(|p| {
            p.x += p.vx * dt_sec;
            p.y += p.vy * dt_sec;
            p.life -= dt_sec * 2.2;
            p.life > 0.0
        });

        let exhausted = self.balloon.next_word_idx >= self.balloon.words.len()
            && self.balloon.balloons.is_empty();
        if self.balloon.lives == 0 || exhausted {
            self.finish_balloon_wave();
        } else {
            self.bump();
        }
    }

    pub fn finish_balloon_wave(&mut self) {
        if self.balloon.completed {
            return;
        }
        self.balloon.completed = true;
        let base_acc = self.metrics.accuracy;
        let acc = (base_acc - (self.balloon.missed as f64) * 5.0).clamp(0.0, 100.0);
        let errors = self.metrics.errors + self.balloon.missed;
        let keys = self.metrics.keystrokes.max(self.balloon.popped * 3);
        self.complete_active_jungle_lesson(Some((self.metrics.net_wpm, acc, errors, keys)));
    }

    pub fn apply_settings_from_modals(&mut self) {
        let mut s = self.engine.settings().clone();
        s.target_wpm = self.modals.target_wpm.get_untracked() as f64;
        s.target_accuracy = self.modals.target_accuracy.get_untracked() as f64;
        s.unlock_threshold = self.modals.unlock_threshold.get_untracked() as f64;
        if self.engine.update_settings(s).is_ok() {
            if let Ok(a) = self.engine.analytics(self.layout) {
                self.key_stats = a.key_stats;
                self.sync_unlock_strip();
            }
            self.persist_snapshot();
            self.bump();
        }
    }

    pub fn export_backup_from_modal(&self) {
        match self.engine.export_backup_json() {
            Ok(json) => {
                if let Some(ref hook) = self.export_hook {
                    match hook(&json) {
                        Ok(dest) => self
                            .modals
                            .settings_status
                            .set(format!("Exported to {dest}")),
                        Err(e) => self.modals.settings_status.set(e),
                    }
                }
            }
            Err(e) => self.modals.settings_status.set(e.to_string()),
        }
    }

    pub fn import_backup_from_modal(&mut self) {
        if let Some(ref hook) = self.import_hook
            && let Some(raw) = hook()
        {
            match self.engine.import_backup_json(&raw, false) {
                Ok(_) => {
                    self.bootstrap();
                    self.modals
                        .settings_status
                        .set("Backup merged successfully.".to_string());
                    self.persist_snapshot();
                }
                Err(e) => self.modals.settings_status.set(e.to_string()),
            }
        }
    }

    pub fn reset_current_layout(&mut self) {
        if let Ok(boot) = self.engine.reset_layout_progress(self.layout) {
            self.key_stats = boot.key_stats;
            self.apply_session_snapshot(boot.session);
            self.sync_metrics_bar();
            self.sync_unlock_strip();
            self.modals
                .settings_status
                .set("Layout progress reset to Home Row.".to_string());
            self.persist_snapshot();
            self.bump();
        }
    }

    pub fn handle_craft_typing_key(&mut self, ke: &KeyEvent) -> bool {
        self.caps_lock = ke.modifiers.caps_lock;
        self.shift_held = ke.modifiers.shift;
        self.alt_gr_held = ke.modifiers.alt_gr;
        if !ke.pressed {
            self.sync_metrics_bar();
            self.bump();
            return false;
        }
        if self.exam_summary.is_some() || self.lesson_completion.is_some() {
            return false;
        }

        if matches!(ke.code, KeyCode::Tab) {
            self.tab_armed_for_restart = true;
            return true;
        }
        if matches!(ke.code, KeyCode::Enter) && self.tab_armed_for_restart {
            self.tab_armed_for_restart = false;
            self.restart_or_exit();
            return true;
        }
        self.tab_armed_for_restart = false;

        if matches!(
            ke.code,
            KeyCode::ShiftLeft
                | KeyCode::ShiftRight
                | KeyCode::ControlLeft
                | KeyCode::ControlRight
                | KeyCode::AltLeft
                | KeyCode::AltRight
                | KeyCode::CapsLock
        ) || ke.modifiers.super_key
            || (ke.modifiers.ctrl && !matches!(ke.code, KeyCode::Backspace))
        {
            self.sync_metrics_bar();
            self.bump();
            return false;
        }

        let Some(dom_code) = resolve_dom_code(ke) else {
            return false;
        };

        let input = KeyInputEvent {
            code: dom_code.to_string(),
            shift: self.shift_held || self.touch_shift,
            alt_gr: self.alt_gr_held || self.touch_alt_gr,
            ctrl: ke.modifiers.ctrl,
            caps_lock: self.caps_lock,
            timestamp_ms: monotonic_ms() as f64,
        };
        if let Ok(delta) = self.engine.process_key(&input) {
            self.touch_shift = false;
            self.touch_alt_gr = false;
            self.apply_delta(delta);
            return true;
        }
        false
    }

    pub fn handle_craft_game_key(&mut self, ke: &KeyEvent) -> bool {
        self.caps_lock = ke.modifiers.caps_lock;
        self.shift_held = ke.modifiers.shift;
        self.alt_gr_held = ke.modifiers.alt_gr;
        if !ke.pressed {
            self.bump();
            return false;
        }
        if self.balloon.paused || self.balloon.completed || self.lesson_completion.is_some() {
            return false;
        }
        if matches!(
            ke.code,
            KeyCode::ShiftLeft
                | KeyCode::ShiftRight
                | KeyCode::ControlLeft
                | KeyCode::ControlRight
                | KeyCode::AltLeft
                | KeyCode::AltRight
                | KeyCode::CapsLock
        ) || ke.modifiers.super_key
            || ke.modifiers.ctrl
        {
            return false;
        }

        let Some(dom_code) = resolve_dom_code(ke) else {
            return false;
        };

        let mut sorted_balloons = self.balloon.balloons.clone();
        sorted_balloons.sort_by(|a, b| {
            b.y_px
                .partial_cmp(&a.y_px)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let candidates: Vec<String> = sorted_balloons.into_iter().map(|b| b.word).collect();

        let input = KeyInputEvent {
            code: dom_code.to_string(),
            shift: self.shift_held || self.touch_shift,
            alt_gr: self.alt_gr_held || self.touch_alt_gr,
            ctrl: ke.modifiers.ctrl,
            caps_lock: self.caps_lock,
            timestamp_ms: monotonic_ms() as f64,
        };
        if let Ok(res) = self.engine.resolve_game_keystroke(&input, candidates) {
            self.touch_shift = false;
            self.touch_alt_gr = false;
            self.metrics = res.metrics;
            self.pending_ime_preview = res.pending_ime_preview;
            self.balloon.input_error = res.is_error;
            if res.popped {
                let target_w = res.locked_word.or_else(|| self.balloon.locked_word.clone());
                if let Some(w) = target_w
                    && let Some(idx) = self.balloon.balloons.iter().position(|b| b.word == w)
                {
                    let hit = self.balloon.balloons.remove(idx);
                    let arena_w = (self.balloon.canvas_width - 24.0).max(1.0);
                    let cx = hit.x_ratio * arena_w;
                    for k in 0..8 {
                        let angle = std::f32::consts::TAU * (k as f32) / 8.0;
                        self.balloon.particles.push(PopParticle {
                            x: cx,
                            y: hit.y_px,
                            vx: angle.cos() * 75.0,
                            vy: angle.sin() * 75.0,
                            life: 1.0,
                        });
                    }
                }
                self.balloon.popped += 1;
                self.balloon.score += 100;
                self.balloon.locked_word = None;
                self.balloon.typed_preview.clear();
            } else {
                self.balloon.locked_word = res.locked_word;
                self.balloon.typed_preview = res.typed_preview;
            }
            self.sync_metrics_bar();
            self.bump();
            return true;
        }
        false
    }

    pub fn press_virtual_key(&mut self, dom_code: &str) {
        if dom_code == "ShiftLeft" || dom_code == "ShiftRight" {
            self.touch_shift = !self.touch_shift;
            self.bump();
            return;
        }
        if dom_code == "AltRight" {
            self.touch_alt_gr = !self.touch_alt_gr;
            self.bump();
            return;
        }
        let Some(kc) = KeyCode::from_dom_code(dom_code) else {
            return;
        };
        let ke = KeyEvent {
            code: kc,
            text: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                shift: self.touch_shift || self.shift_held,
                alt_gr: self.touch_alt_gr || self.alt_gr_held,
                caps_lock: self.caps_lock,
                ..Default::default()
            },
        };
        if self.mode == PracticeMode::MiniGame {
            self.handle_craft_game_key(&ke);
        } else {
            self.handle_craft_typing_key(&ke);
        }
    }
}

fn top_weak_keys(stats: &[KeyStatRecord], n: usize) -> Vec<KeyStatRecord> {
    let mut unlocked: Vec<KeyStatRecord> = stats.iter().filter(|k| k.unlocked).cloned().collect();
    let practiced_any = unlocked.iter().any(|k| k.hits + k.misses > 0);
    if practiced_any {
        unlocked.retain(|k| k.hits + k.misses > 0);
    }
    unlocked.sort_by(|a, b| {
        b.weakness_weight
            .partial_cmp(&a.weakness_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    unlocked.truncate(n);
    unlocked
}

fn resolve_dom_code(ke: &KeyEvent) -> Option<&'static str> {
    if let Some(code) = ke.code.dom_code() {
        return Some(code);
    }
    if let KeyCode::Character(ref s) = ke.code
        && s.len() == 1
        && let Some(ch) = s.chars().next()
    {
        return match ch.to_ascii_lowercase() {
            'a' => Some("KeyA"),
            'b' => Some("KeyB"),
            'c' => Some("KeyC"),
            'd' => Some("KeyD"),
            'e' => Some("KeyE"),
            'f' => Some("KeyF"),
            'g' => Some("KeyG"),
            'h' => Some("KeyH"),
            'i' => Some("KeyI"),
            'j' => Some("KeyJ"),
            'k' => Some("KeyK"),
            'l' => Some("KeyL"),
            'm' => Some("KeyM"),
            'n' => Some("KeyN"),
            'o' => Some("KeyO"),
            'p' => Some("KeyP"),
            'q' => Some("KeyQ"),
            'r' => Some("KeyR"),
            's' => Some("KeyS"),
            't' => Some("KeyT"),
            'u' => Some("KeyU"),
            'v' => Some("KeyV"),
            'w' => Some("KeyW"),
            'x' => Some("KeyX"),
            'y' => Some("KeyY"),
            'z' => Some("KeyZ"),
            '0' => Some("Digit0"),
            '1' => Some("Digit1"),
            '2' => Some("Digit2"),
            '3' => Some("Digit3"),
            '4' => Some("Digit4"),
            '5' => Some("Digit5"),
            '6' => Some("Digit6"),
            '7' => Some("Digit7"),
            '8' => Some("Digit8"),
            '9' => Some("Digit9"),
            '-' => Some("Minus"),
            '=' => Some("Equal"),
            '[' => Some("BracketLeft"),
            ']' => Some("BracketRight"),
            '\\' => Some("Backslash"),
            ';' => Some("Semicolon"),
            '\'' => Some("Quote"),
            ',' => Some("Comma"),
            '.' => Some("Period"),
            '/' => Some("Slash"),
            '`' => Some("Backquote"),
            ' ' => Some("Space"),
            _ => None,
        };
    }
    None
}

fn stage_matches(idx: usize, stage: CurriculumStage) -> bool {
    match idx {
        1 => stage == CurriculumStage::HomeRow,
        2 => stage == CurriculumStage::TopRowAndMatras,
        3 => stage == CurriculumStage::BottomRowAndConjuncts,
        4 => stage == CurriculumStage::ShiftNumbersAndSymbols,
        5 => stage == CurriculumStage::FluencyAndStories,
        _ => true,
    }
}
