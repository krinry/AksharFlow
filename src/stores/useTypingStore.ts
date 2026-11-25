import { create } from "zustand";
import { open as openFileDialog, save as saveFileDialog } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type ActiveLessonMeta,
  type AnalyticsSnapshot_Serialize,
  type AppSettings,
  type BootstrapSnapshot_Serialize,
  type ExamPassage_Serialize,
  type GameKeyResult_Serialize,
  type GraphemeStatus,
  type JungleRoadmapSnapshot,
  type KeyEventDelta_Serialize,
  type KeyInputEvent,
  type KeyStatRecord_Serialize,
  type KeycapDescriptor_Serialize,
  type LayoutKind,
  type LessonCompletionResult_Serialize,
  type LiveMetrics,
  type NextKeyHint,
  type PracticeMode,
  type SessionRecord,
  type SessionSnapshot_Serialize,
  type SessionSummary,
  type UnlockedKeyEvent,
} from "@/lib/bindings";
import { soundEngine, type SoundProfile } from "@/audio/soundEngine";
import type { UiLanguage } from "@/i18n/useTranslation";

export type ThemeMode = "dark" | "light";
export type NavTab = "jungle" | "adaptive" | "exam" | "analytics";

const INITIAL_METRICS: LiveMetrics = {
  gross_wpm: 0,
  net_wpm: 0,
  kph: 0,
  accuracy: 100,
  consistency: 100,
  elapsed_ms: 0,
  keystrokes: 0,
  errors: 0,
};

const DEFAULT_SETTINGS: AppSettings = {
  ui_lang: "en",
  theme: "dark",
  sound_profile: "brown",
  sound_volume: 0.55,
  active_layout: "en_qwerty",
  target_wpm: 25,
  target_accuracy: 95,
  unlock_threshold: 85,
  zen_mode: false,
  updated_at: "",
};

export interface TypingState {
  activeTab: NavTab;
  layout: LayoutKind;
  practiceMode: PracticeMode;
  uiLang: UiLanguage;
  theme: ThemeMode;
  soundProfile: SoundProfile;
  soundVolume: number;
  zenMode: boolean;
  showHeatmap: boolean;
  settings: AppSettings;
  settingsModalOpen: boolean;

  capsLock: boolean;
  shiftHeld: boolean;
  altGrHeld: boolean;
  touchShift: boolean;
  touchAltGr: boolean;

  sessionId: string;
  lineIndex: number;
  graphemes: string[];
  statuses: GraphemeStatus[];
  graphemeIndex: number;
  activeTypedPreview: string;
  activeIsPartial: boolean;
  activeHasError: boolean;
  pendingImePreview: string | null;
  nextKeyHint: NextKeyHint | null;
  metrics: LiveMetrics;
  keycaps: KeycapDescriptor_Serialize[];
  keyStats: KeyStatRecord_Serialize[];
  recentSessions: SessionRecord[];
  unlockedToast: UnlockedKeyEvent | null;

  // Exam Mode
  examPassages: ExamPassage_Serialize[];
  selectedPassageId: string;
  examDurationMins: number;
  examAllowBackspace: boolean;
  examRemainingSecs: number;
  examTimerRunning: boolean;
  examSummary: SessionSummary | null;

  // Jungle Journey & Mini-Game
  jungleRoadmap: JungleRoadmapSnapshot | null;
  roadmapLoading: boolean;
  roadmapError: string | null;
  activeLessonMeta: ActiveLessonMeta | null;
  miniGameWords: string[];
  lessonCompletion: LessonCompletionResult_Serialize | null;

  // Analytics
  analytics: AnalyticsSnapshot_Serialize | null;
  analyticsLoading: boolean;

  isReady: boolean;
  errorMessage: string | null;

  initApp: () => Promise<void>;
  initSession: (layout?: LayoutKind) => Promise<void>;
  setActiveTab: (tab: NavTab) => Promise<void>;
  setLayout: (layout: LayoutKind) => Promise<void>;
  setUiLang: (lang: UiLanguage) => void;
  toggleTheme: () => void;
  setSoundProfile: (profile: SoundProfile) => void;
  setSoundVolume: (vol: number) => void;
  toggleZenMode: () => void;
  toggleHeatmap: () => void;
  setSettingsModalOpen: (open: boolean) => void;
  updateAdaptiveSettings: (patch: Partial<AppSettings>) => Promise<void>;
  exportProgressBackup: () => Promise<string | null>;
  importProgressBackup: (replace: boolean) => Promise<boolean>;
  resetCurrentLayout: () => Promise<void>;
  dismissUnlockToast: () => void;

  // Jungle Journey actions
  loadJungleRoadmap: (layout?: LayoutKind) => Promise<void>;
  startJungleLesson: (lessonId: string) => Promise<void>;
  restartActiveLesson: () => Promise<void>;
  exitActiveLessonToMap: () => void;
  completeJungleLesson: (overrideStats?: {
    netWpm?: number | null;
    accuracy?: number | null;
    errors?: number | null;
    keystrokes?: number | null;
  }) => Promise<void>;
  dismissLessonCompletion: () => void;
  resolveGameKeystroke: (
    event: KeyInputEvent,
    candidateWords: string[],
  ) => Promise<GameKeyResult_Serialize | null>;

  // Exam & Drill actions
  loadExamPassages: () => Promise<void>;
  setExamConfig: (patch: {
    selectedPassageId?: string;
    examDurationMins?: number;
    examAllowBackspace?: boolean;
  }) => void;
  startExamWithSelected: (overrideContent?: string) => Promise<void>;
  saveCustomPassage: (title: string, language: string, content: string) => Promise<void>;
  tickExamTimer: () => void;
  finishExam: () => Promise<void>;
  dismissExamSummary: () => void;
  startWeakKeyDrill: (focusKeys: string[]) => Promise<void>;
  refreshAnalytics: () => Promise<void>;

  setModifiers: (mods: { capsLock?: boolean; shiftHeld?: boolean; altGrHeld?: boolean }) => void;
  toggleTouchShift: () => void;
  toggleTouchAltGr: () => void;
  enqueueKeyInput: (event: KeyInputEvent) => Promise<void>;
  applyBootstrap: (boot: BootstrapSnapshot_Serialize) => void;
  applySnapshot: (snap: SessionSnapshot_Serialize) => void;
  applyDelta: (delta: KeyEventDelta_Serialize) => void;
}

let keyQueueTail: Promise<void> = Promise.resolve();

export function resetKeyQueueForTest(): void {
  keyQueueTail = Promise.resolve();
}

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function persistSettingsSilently(settings: AppSettings): void {
  if (!isTauriRuntime()) return;
  void commands.updateSettings(settings).catch((err) => {
    useTypingStore.setState({ errorMessage: String(err) });
  });
}

function defaultPassageForLayout(
  passages: ExamPassage_Serialize[],
  layout: LayoutKind,
): ExamPassage_Serialize | undefined {
  const targetLang = layout === "en_qwerty" ? "en" : "hi";
  return passages.find((p) => p.language === targetLang) ?? passages[0];
}

export const useTypingStore = create<TypingState>((set, get) => ({
  activeTab: "adaptive",
  layout: "en_qwerty",
  practiceMode: "adaptive",
  uiLang: "en",
  theme: "dark",
  soundProfile: "brown",
  soundVolume: 0.55,
  zenMode: false,
  showHeatmap: true,
  settings: DEFAULT_SETTINGS,
  settingsModalOpen: false,

  capsLock: false,
  shiftHeld: false,
  altGrHeld: false,
  touchShift: false,
  touchAltGr: false,

  sessionId: "",
  lineIndex: 0,
  graphemes: [],
  statuses: [],
  graphemeIndex: 0,
  activeTypedPreview: "",
  activeIsPartial: false,
  activeHasError: false,
  pendingImePreview: null,
  nextKeyHint: null,
  metrics: INITIAL_METRICS,
  keycaps: [],
  keyStats: [],
  recentSessions: [],
  unlockedToast: null,

  examPassages: [],
  selectedPassageId: "",
  examDurationMins: 5,
  examAllowBackspace: true,
  examRemainingSecs: 300,
  examTimerRunning: false,
  examSummary: null,

  jungleRoadmap: null,
  roadmapLoading: false,
  roadmapError: null,
  activeLessonMeta: null,
  miniGameWords: [],
  lessonCompletion: null,

  analytics: null,
  analyticsLoading: false,

  isReady: false,
  errorMessage: null,

  initApp: async () => {
    if (!isTauriRuntime()) {
      set({
        isReady: false,
        errorMessage: "Tauri IPC unavailable: launch with pnpm tauri dev.",
      });
      return;
    }
    try {
      const [boot, passages] = await Promise.all([
        commands.getBootstrapState(18),
        commands.listExamPassages(),
      ]);
      get().applyBootstrap(boot);
      const preferred = defaultPassageForLayout(passages, boot.settings.active_layout);
      set({
        examPassages: passages,
        selectedPassageId: preferred?.id ?? "",
      });
    } catch (err) {
      set({ errorMessage: String(err), isReady: false });
    }
  },

  initSession: async (overrideLayout) => {
    const targetLayout = overrideLayout ?? get().layout;
    if (!isTauriRuntime()) {
      set({
        isReady: false,
        errorMessage: "Tauri IPC unavailable: launch with pnpm tauri dev.",
      });
      return;
    }
    try {
      const boot = await commands.startSession(targetLayout, 18);
      get().applyBootstrap(boot);
      set({ examTimerRunning: false, examSummary: null, lessonCompletion: null });
    } catch (err) {
      set({ errorMessage: String(err), isReady: false });
    }
  },

  setActiveTab: async (tab) => {
    const prevTab = get().activeTab;
    const prevMode = get().practiceMode;
    set({ activeTab: tab, lessonCompletion: null });

    if (tab === "analytics") {
      await get().refreshAnalytics();
    } else if (tab === "exam") {
      set({ activeLessonMeta: null, miniGameWords: [] });
      await get().loadExamPassages();
      if (get().practiceMode !== "exam") {
        await get().startExamWithSelected();
      }
    } else if (tab === "jungle") {
      set({
        examTimerRunning: false,
        examSummary: null,
        ...(prevTab === "jungle" ? { activeLessonMeta: null, miniGameWords: [] } : {}),
      });
      if (!get().jungleRoadmap || prevTab === "jungle" || get().activeLessonMeta === null) {
        await get().loadJungleRoadmap();
      }
    } else if (tab === "adaptive") {
      set({
        examTimerRunning: false,
        examSummary: null,
        activeLessonMeta: null,
        miniGameWords: [],
      });
      if (prevMode !== "adaptive" && prevMode !== "weak_drill") {
        await get().initSession(get().layout);
      }
    }
  },

  setLayout: async (layout) => {
    const passages = get().examPassages;
    const preferred = defaultPassageForLayout(passages, layout);
    const activeTab = get().activeTab;
    set({
      layout,
      touchShift: false,
      touchAltGr: false,
      unlockedToast: null,
      selectedPassageId: preferred?.id ?? get().selectedPassageId,
      examTimerRunning: false,
      examSummary: null,
      activeLessonMeta: null,
      miniGameWords: [],
      lessonCompletion: null,
    });
    if (activeTab === "exam") {
      await get().startExamWithSelected(preferred?.content);
    } else if (activeTab === "jungle") {
      await get().loadJungleRoadmap(layout);
    } else {
      await get().initSession(layout);
    }
    if (activeTab === "analytics") {
      await get().refreshAnalytics();
    }
  },

  setUiLang: (uiLang) => {
    const next = { ...get().settings, ui_lang: uiLang };
    set({ uiLang, settings: next });
    persistSettingsSilently(next);
  },

  toggleTheme: () => {
    const theme: ThemeMode = get().theme === "dark" ? "light" : "dark";
    const next = { ...get().settings, theme };
    set({ theme, settings: next });
    persistSettingsSilently(next);
  },

  setSoundProfile: (soundProfile) => {
    const next = { ...get().settings, sound_profile: soundProfile };
    set({ soundProfile, settings: next });
    persistSettingsSilently(next);
  },

  setSoundVolume: (soundVolume) => {
    const next = { ...get().settings, sound_volume: soundVolume };
    set({ soundVolume, settings: next });
    persistSettingsSilently(next);
  },

  toggleZenMode: () => {
    const zenMode = !get().zenMode;
    const next = { ...get().settings, zen_mode: zenMode };
    set({ zenMode, settings: next });
    persistSettingsSilently(next);
  },

  toggleHeatmap: () => set((s) => ({ showHeatmap: !s.showHeatmap })),

  setSettingsModalOpen: (settingsModalOpen) => set({ settingsModalOpen }),

  updateAdaptiveSettings: async (patch) => {
    const merged: AppSettings = { ...get().settings, ...patch };
    set({
      settings: merged,
      soundVolume: merged.sound_volume ?? get().soundVolume,
    });
    if (!isTauriRuntime()) return;
    const saved = await commands.updateSettings(merged);
    const analytics = await commands.getAnalytics(get().layout);
    set({ settings: saved, keyStats: analytics.key_stats, analytics });
  },

  exportProgressBackup: async () => {
    if (!isTauriRuntime()) return null;
    const path = await saveFileDialog({
      title: "Export AksharFlow Progress",
      defaultPath: `aksharflow-backup-${new Date().toISOString().slice(0, 10)}.json`,
      filters: [{ name: "JSON Backup", extensions: ["json"] }],
    });
    if (!path) return null;
    const result = await commands.exportBackupFile(path);
    return result.path;
  },

  importProgressBackup: async (replace) => {
    if (!isTauriRuntime()) return false;
    const selected = await openFileDialog({
      title: "Import AksharFlow Progress",
      multiple: false,
      filters: [{ name: "JSON Backup", extensions: ["json"] }],
    });
    if (!selected || typeof selected !== "string") return false;
    const boot = await commands.importBackupFile(selected, replace);
    get().applyBootstrap(boot);
    await get().loadExamPassages();
    if (get().activeTab === "analytics") {
      await get().refreshAnalytics();
    }
    return true;
  },

  resetCurrentLayout: async () => {
    if (!isTauriRuntime()) return;
    const boot = await commands.resetLayoutProgress(get().layout);
    get().applyBootstrap(boot);
    if (get().activeTab === "analytics") {
      await get().refreshAnalytics();
    }
  },

  dismissUnlockToast: () => set({ unlockedToast: null }),

  loadExamPassages: async () => {
    if (!isTauriRuntime()) return;
    const passages = await commands.listExamPassages();
    const currentId = get().selectedPassageId;
    const exists = passages.some((p) => p.id === currentId);
    const fallback = defaultPassageForLayout(passages, get().layout);
    set({
      examPassages: passages,
      selectedPassageId: exists ? currentId : (fallback?.id ?? ""),
    });
  },

  setExamConfig: (patch) => {
    set((s) => ({
      selectedPassageId: patch.selectedPassageId ?? s.selectedPassageId,
      examDurationMins: patch.examDurationMins ?? s.examDurationMins,
      examAllowBackspace: patch.examAllowBackspace ?? s.examAllowBackspace,
    }));
  },

  startExamWithSelected: async (overrideContent) => {
    if (!isTauriRuntime()) return;
    const state = get();
    const passage =
      state.examPassages.find((p) => p.id === state.selectedPassageId) ??
      defaultPassageForLayout(state.examPassages, state.layout);
    const content = overrideContent ?? passage?.content ?? "";
    if (!content.trim()) return;

    const durationSecs = state.examDurationMins * 60;
    const snap = await commands.startExamSession(
      state.layout,
      content,
      durationSecs,
      state.examAllowBackspace,
    );
    get().applySnapshot(snap);
    set({
      examRemainingSecs: durationSecs,
      examTimerRunning: false,
      examSummary: null,
    });
  },

  saveCustomPassage: async (title, language, content) => {
    if (!isTauriRuntime()) return;
    const saved = await commands.saveCustomPassage(title, language, content);
    await get().loadExamPassages();
    set({ selectedPassageId: saved.id });
    if (get().activeTab === "exam") {
      await get().startExamWithSelected(saved.content);
    }
  },

  tickExamTimer: () => {
    const state = get();
    if (!state.examTimerRunning || state.practiceMode !== "exam") return;
    if (state.examRemainingSecs <= 1) {
      set({ examRemainingSecs: 0, examTimerRunning: false });
      void state.finishExam();
    } else {
      set({ examRemainingSecs: state.examRemainingSecs - 1 });
    }
  },

  finishExam: async () => {
    if (!isTauriRuntime()) return;
    set({ examTimerRunning: false });
    try {
      const summary = await commands.finishSession();
      set({ examSummary: summary });
    } catch (err) {
      set({ errorMessage: String(err) });
    }
  },

  dismissExamSummary: () => set({ examSummary: null }),

  startWeakKeyDrill: async (focusKeys) => {
    if (!isTauriRuntime()) return;
    const boot = await commands.startWeakKeyDrill(get().layout, focusKeys, 18);
    get().applyBootstrap(boot);
    set({
      activeTab: "adaptive",
      examTimerRunning: false,
      examSummary: null,
      activeLessonMeta: null,
      miniGameWords: [],
      lessonCompletion: null,
    });
  },

  loadJungleRoadmap: async (overrideLayout) => {
    const targetLayout = overrideLayout ?? get().layout;
    if (!isTauriRuntime()) {
      set({
        roadmapLoading: false,
        roadmapError: "Tauri IPC unavailable: launch with pnpm tauri dev.",
      });
      return;
    }
    set({ roadmapLoading: true, roadmapError: null });
    try {
      const jungleRoadmap = await commands.getJungleRoadmap(targetLayout);
      set({ jungleRoadmap, roadmapLoading: false, roadmapError: null });
    } catch (err) {
      set({ roadmapLoading: false, roadmapError: String(err) });
    }
  },

  startJungleLesson: async (lessonId) => {
    if (!isTauriRuntime()) return;
    set({
      examTimerRunning: false,
      examSummary: null,
      lessonCompletion: null,
      errorMessage: null,
    });
    try {
      const snap = await commands.startJungleLesson(get().layout, lessonId);
      set({ activeTab: "jungle" });
      get().applySnapshot(snap);
    } catch (err) {
      set({ errorMessage: String(err) });
    }
  },

  restartActiveLesson: async () => {
    const meta = get().activeLessonMeta;
    if (meta) {
      await get().startJungleLesson(meta.id);
    } else {
      await get().initSession();
    }
  },

  exitActiveLessonToMap: () => {
    set({
      activeTab: "jungle",
      activeLessonMeta: null,
      miniGameWords: [],
      lessonCompletion: null,
      practiceMode: "adaptive",
    });
    void get().loadJungleRoadmap();
  },

  completeJungleLesson: async (overrideStats) => {
    if (!isTauriRuntime()) return;
    try {
      const result = await commands.completeJungleLesson(
        overrideStats?.netWpm ?? null,
        overrideStats?.accuracy ?? null,
        overrideStats?.errors ?? null,
        overrideStats?.keystrokes ?? null,
      );
      set({ lessonCompletion: result });
      void get().loadJungleRoadmap();
    } catch (err) {
      set({ errorMessage: String(err) });
    }
  },

  dismissLessonCompletion: () => set({ lessonCompletion: null }),

  resolveGameKeystroke: (event, candidateWords) => {
    let result: GameKeyResult_Serialize | null = null;
    const run = async () => {
      try {
        const res = await commands.resolveGameKeystroke(event, candidateWords);
        result = res;
        const state = get();
        soundEngine.play(res.sound_kind, state.soundProfile, state.soundVolume);
        set({
          metrics: res.metrics,
          pendingImePreview: res.pending_ime_preview,
          errorMessage: null,
        });
      } catch (err) {
        set({ errorMessage: String(err) });
      }
    };
    const next = keyQueueTail.then(run, run);
    keyQueueTail = next;
    return next.then(() => result);
  },

  refreshAnalytics: async () => {
    if (!isTauriRuntime()) return;
    set({ analyticsLoading: true });
    try {
      const analytics = await commands.getAnalytics(get().layout);
      set({ analytics, analyticsLoading: false });
    } catch (err) {
      set({ errorMessage: String(err), analyticsLoading: false });
    }
  },

  setModifiers: ({ capsLock, shiftHeld, altGrHeld }) =>
    set((s) => ({
      capsLock: capsLock ?? s.capsLock,
      shiftHeld: shiftHeld ?? s.shiftHeld,
      altGrHeld: altGrHeld ?? s.altGrHeld,
    })),

  toggleTouchShift: () => set((s) => ({ touchShift: !s.touchShift })),

  toggleTouchAltGr: () => set((s) => ({ touchAltGr: !s.touchAltGr })),

  applyBootstrap: (boot) => {
    const s = boot.settings;
    set({
      settings: s,
      uiLang: s.ui_lang === "hi" ? "hi" : "en",
      theme: s.theme === "light" ? "light" : "dark",
      soundProfile: (s.sound_profile as SoundProfile) || "brown",
      soundVolume: s.sound_volume ?? 0.55,
      zenMode: s.zen_mode,
      keyStats: boot.key_stats,
      recentSessions: boot.recent_sessions,
    });
    get().applySnapshot(boot.session);
  },

  applySnapshot: (snap) => {
    set({
      sessionId: snap.session_id,
      layout: snap.layout,
      practiceMode: snap.mode,
      activeLessonMeta: snap.lesson_meta ?? null,
      miniGameWords: snap.words ?? [],
      lineIndex: snap.line_index,
      graphemes: snap.graphemes,
      statuses: snap.graphemes.map(() => "pending"),
      graphemeIndex: snap.delta.grapheme_index,
      activeTypedPreview: snap.delta.active_typed_preview,
      activeIsPartial: snap.delta.active_is_partial,
      activeHasError: snap.delta.active_has_error,
      pendingImePreview: snap.delta.pending_ime_preview,
      nextKeyHint: snap.delta.next_key_hint,
      metrics: snap.delta.metrics,
      keycaps: snap.keycaps,
      isReady: true,
      errorMessage: null,
    });
  },

  applyDelta: (delta) => {
    const state = get();
    soundEngine.play(
      delta.newly_unlocked_key ? "line_complete" : delta.sound_kind,
      state.soundProfile,
      state.soundVolume,
    );

    const shouldStartExamTimer =
      state.practiceMode === "exam" &&
      !state.examTimerRunning &&
      state.examRemainingSecs > 0 &&
      delta.metrics.keystrokes > 0 &&
      !state.examSummary;

    const shouldFinishJungle =
      state.practiceMode === "jungle" &&
      delta.line_completed &&
      !delta.next_line &&
      state.activeLessonMeta !== null &&
      !state.lessonCompletion;

    let nextStatuses = state.statuses;
    let nextLineIndex = state.lineIndex;
    let nextGraphemes = state.graphemes;

    if (delta.line_completed && delta.next_line) {
      nextLineIndex = delta.next_line.line_index;
      nextGraphemes = delta.next_line.graphemes;
      nextStatuses = delta.next_line.graphemes.map(() => "pending");
    } else if (delta.committed_updates.length > 0) {
      nextStatuses = state.statuses.slice();
      for (const patch of delta.committed_updates) {
        if (patch.index >= 0 && patch.index < nextStatuses.length) {
          nextStatuses[patch.index] = patch.status;
        }
      }
    }

    set({
      lineIndex: nextLineIndex,
      graphemes: nextGraphemes,
      statuses: nextStatuses,
      graphemeIndex: delta.grapheme_index,
      activeTypedPreview: delta.active_typed_preview,
      activeIsPartial: delta.active_is_partial,
      activeHasError: delta.active_has_error,
      pendingImePreview: delta.pending_ime_preview,
      nextKeyHint: delta.next_key_hint,
      metrics: delta.metrics,
      keyStats: delta.updated_key_stats ?? state.keyStats,
      unlockedToast: delta.newly_unlocked_key ?? state.unlockedToast,
      examTimerRunning: shouldStartExamTimer ? true : state.examTimerRunning,
    });

    if (shouldFinishJungle) {
      void get().completeJungleLesson();
    }
  },

  enqueueKeyInput: (event) => {
    const run = async () => {
      try {
        const delta = await commands.processKeyEvent(event);
        get().applyDelta(delta);
      } catch (err) {
        set({ errorMessage: String(err) });
      }
    };
    const next = keyQueueTail.then(run, run);
    keyQueueTail = next;
    return next;
  },
}));
