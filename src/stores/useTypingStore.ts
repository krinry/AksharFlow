import { create } from "zustand";
import {
  commands,
  type GraphemeStatus,
  type KeyEventDelta_Serialize,
  type KeyInputEvent,
  type KeycapDescriptor_Serialize,
  type LayoutKind,
  type LiveMetrics,
  type NextKeyHint,
  type SessionSnapshot_Serialize,
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

export interface TypingState {
  layout: LayoutKind;
  uiLang: UiLanguage;
  theme: ThemeMode;
  soundProfile: SoundProfile;
  soundVolume: number;
  zenMode: boolean;

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

  isReady: boolean;
  errorMessage: string | null;

  initSession: (layout?: LayoutKind) => Promise<void>;
  setLayout: (layout: LayoutKind) => Promise<void>;
  setUiLang: (lang: UiLanguage) => void;
  toggleTheme: () => void;
  setSoundProfile: (profile: SoundProfile) => void;
  setSoundVolume: (vol: number) => void;
  toggleZenMode: () => void;
  setModifiers: (mods: { capsLock?: boolean; shiftHeld?: boolean; altGrHeld?: boolean }) => void;
  toggleTouchShift: () => void;
  toggleTouchAltGr: () => void;
  enqueueKeyInput: (event: KeyInputEvent) => Promise<void>;
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

export const useTypingStore = create<TypingState>((set, get) => ({
  layout: "en_qwerty",
  uiLang: "en",
  theme: "dark",
  soundProfile: "brown",
  soundVolume: 0.55,
  zenMode: false,

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

  isReady: false,
  errorMessage: null,

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
      const snap = await commands.startSession(targetLayout, 18);
      get().applySnapshot(snap);
    } catch (err) {
      set({ errorMessage: String(err), isReady: false });
    }
  },

  setLayout: async (layout) => {
    set({ layout, touchShift: false, touchAltGr: false });
    await get().initSession(layout);
  },

  setUiLang: (uiLang) => set({ uiLang }),

  toggleTheme: () =>
    set((s) => ({ theme: s.theme === "dark" ? "light" : "dark" })),

  setSoundProfile: (soundProfile) => set({ soundProfile }),

  setSoundVolume: (soundVolume) => set({ soundVolume }),

  toggleZenMode: () => set((s) => ({ zenMode: !s.zenMode })),

  setModifiers: ({ capsLock, shiftHeld, altGrHeld }) =>
    set((s) => ({
      capsLock: capsLock ?? s.capsLock,
      shiftHeld: shiftHeld ?? s.shiftHeld,
      altGrHeld: altGrHeld ?? s.altGrHeld,
    })),

  toggleTouchShift: () => set((s) => ({ touchShift: !s.touchShift })),

  toggleTouchAltGr: () => set((s) => ({ touchAltGr: !s.touchAltGr })),

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
    soundEngine.play(delta.sound_kind, state.soundProfile, state.soundVolume);

    if (delta.line_completed && delta.next_line) {
      set({
        lineIndex: delta.next_line.line_index,
        graphemes: delta.next_line.graphemes,
        statuses: delta.next_line.graphemes.map(() => "pending"),
        graphemeIndex: delta.grapheme_index,
        activeTypedPreview: delta.active_typed_preview,
        activeIsPartial: delta.active_is_partial,
        activeHasError: delta.active_has_error,
        pendingImePreview: delta.pending_ime_preview,
        nextKeyHint: delta.next_key_hint,
        metrics: delta.metrics,
      });
      return;
    }

    let nextStatuses = state.statuses;
    if (delta.committed_updates.length > 0) {
      nextStatuses = state.statuses.slice();
      for (const patch of delta.committed_updates) {
        if (patch.index >= 0 && patch.index < nextStatuses.length) {
          nextStatuses[patch.index] = patch.status;
        }
      }
    }

    set({
      statuses: nextStatuses,
      graphemeIndex: delta.grapheme_index,
      activeTypedPreview: delta.active_typed_preview,
      activeIsPartial: delta.active_is_partial,
      activeHasError: delta.active_has_error,
      pendingImePreview: delta.pending_ime_preview,
      nextKeyHint: delta.next_key_hint,
      metrics: delta.metrics,
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
