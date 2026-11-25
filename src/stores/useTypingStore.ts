import { create } from "zustand";
import { open as openFileDialog, save as saveFileDialog } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type AppSettings,
  type BootstrapSnapshot_Serialize,
  type GraphemeStatus,
  type KeyEventDelta_Serialize,
  type KeyInputEvent,
  type KeyStatRecord_Serialize,
  type KeycapDescriptor_Serialize,
  type LayoutKind,
  type LiveMetrics,
  type NextKeyHint,
  type SessionRecord,
  type SessionSnapshot_Serialize,
  type UnlockedKeyEvent,
} from "@/lib/bindings";
import { soundEngine, type SoundProfile } from "@/audio/soundEngine";
import type { UiLanguage } from "@/i18n/useTranslation";

export type ThemeMode = "dark" | "light";

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
  layout: LayoutKind;
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

  isReady: boolean;
  errorMessage: string | null;

  initApp: () => Promise<void>;
  initSession: (layout?: LayoutKind) => Promise<void>;
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
  setModifiers: (mods: { capsLock?: boolean; shiftHeld?: boolean; altGrHeld?: boolean }) => void;
  toggleTouchShift: () => void;
  toggleTouchAltGr: () => void;
  enqueueKeyInput: (event: KeyInputEvent) => Promise<void>;
  applyBootstrap: (boot: BootstrapSnapshot_Serialize) => void;
  applySnapshot: (snap: SessionSnapshot_Serialize) => void;
  applyDelta: (delta: KeyEventDelta_Serialize) => void;
}

// Strict sequential FIFO queue guarantees rapid key bursts arrive at Rust in exact timestamp order.
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

export const useTypingStore = create<TypingState>((set, get) => ({
  layout: "en_qwerty",
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

  isReady: false,
  errorMessage: null,

  initApp: async () => {
    if (!isTauriRuntime()) {
      set({
        isReady: false,
        errorMessage: "Tauri IPC unavailable — launch with `pnpm tauri dev`.",
      });
      return;
    }
    try {
      const boot = await commands.getBootstrapState(18);
      get().applyBootstrap(boot);
    } catch (err) {
      set({ errorMessage: String(err), isReady: false });
    }
  },

  initSession: async (overrideLayout) => {
    const targetLayout = overrideLayout ?? get().layout;
    if (!isTauriRuntime()) {
      set({
        isReady: false,
        errorMessage: "Tauri IPC unavailable — launch with `pnpm tauri dev`.",
      });
      return;
    }
    try {
      const boot = await commands.startSession(targetLayout, 18);
      get().applyBootstrap(boot);
    } catch (err) {
      set({ errorMessage: String(err), isReady: false });
    }
  },

  setLayout: async (layout) => {
    set({ layout, touchShift: false, touchAltGr: false, unlockedToast: null });
    await get().initSession(layout);
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
    set({ settings: saved, keyStats: analytics.key_stats });
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
    return true;
  },

  resetCurrentLayout: async () => {
    if (!isTauriRuntime()) return;
    const boot = await commands.resetLayoutProgress(get().layout);
    get().applyBootstrap(boot);
  },

  dismissUnlockToast: () => set({ unlockedToast: null }),

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
    });
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
