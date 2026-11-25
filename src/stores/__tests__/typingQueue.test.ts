import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands, type KeyEventDelta_Serialize, type SessionSnapshot_Serialize } from "@/lib/bindings";
import { resetKeyQueueForTest, useTypingStore } from "@/stores/useTypingStore";

function makeSnapshot(
  graphemes: string[],
  mode: "adaptive" | "weak_drill" | "exam" | "jungle" | "mini_game" = "adaptive",
): SessionSnapshot_Serialize {
  return {
    session_id: "test-1",
    layout: "en_qwerty",
    mode,
    allow_backspace: true,
    exam_duration_secs: mode === "exam" ? 60 : null,
    lesson_meta:
      mode === "jungle" || mode === "mini_game"
        ? {
            id: "u1-l1",
            unit_number: 1,
            lesson_in_unit: 1,
            global_number: 1,
            kind: mode === "mini_game" ? "mini_game" : "key_intro",
            title_en: "Key Introduction · U1.1",
            title_hi: "नई कुंजी परिचय · 1.1",
            focus_keys: ["f", "j"],
            target_wpm: 16,
            min_accuracy: 85,
          }
        : null,
    line_index: 0,
    words: [graphemes.join("")],
    graphemes,
    keycaps: [],
    delta: {
      grapheme_index: 0,
      active_typed_preview: "",
      active_is_partial: false,
      active_has_error: false,
      committed_updates: [],
      pending_ime_preview: null,
      next_key_hint: null,
      metrics: {
        gross_wpm: 0,
        net_wpm: 0,
        kph: 0,
        accuracy: 100,
        consistency: 100,
        elapsed_ms: 0,
        keystrokes: 0,
        errors: 0,
      },
      sound_kind: "none",
      line_completed: false,
      next_line: null,
      newly_unlocked_key: null,
      updated_key_stats: null,
    },
  };
}

function makeDelta(index: number, committedIndex: number): KeyEventDelta_Serialize {
  return {
    grapheme_index: index,
    active_typed_preview: "",
    active_is_partial: false,
    active_has_error: false,
    committed_updates: [{ index: committedIndex, status: "matched" }],
    pending_ime_preview: null,
    next_key_hint: null,
    metrics: {
      gross_wpm: 40,
      net_wpm: 40,
      kph: 12000,
      accuracy: 100,
      consistency: 100,
      elapsed_ms: 500,
      keystrokes: index,
      errors: 0,
    },
    sound_kind: "none",
    line_completed: false,
    next_line: null,
    newly_unlocked_key: null,
    updated_key_stats: null,
  };
}

describe("useTypingStore safeguards, adaptive state & exam timer", () => {
  beforeEach(() => {
    resetKeyQueueForTest();
    useTypingStore.getState().applySnapshot(makeSnapshot(["a", "b", "c"]));
    useTypingStore.setState({
      soundProfile: "off",
      unlockedToast: null,
      keyStats: [],
      examRemainingSecs: 60,
      examTimerRunning: false,
      examSummary: null,
    });
  });

  it("serializes rapid keystrokes strictly in FIFO order", async () => {
    const order: string[] = [];
    vi.spyOn(commands, "processKeyEvent").mockImplementation(async (evt) => {
      if (evt.code === "KeyA") {
        await new Promise((r) => setTimeout(r, 25));
      }
      order.push(evt.code);
      return makeDelta(order.length, order.length - 1);
    });

    const p1 = useTypingStore.getState().enqueueKeyInput({
      code: "KeyA",
      shift: false,
      alt_gr: false,
      ctrl: false,
      caps_lock: false,
      timestamp_ms: 100,
    });
    const p2 = useTypingStore.getState().enqueueKeyInput({
      code: "KeyB",
      shift: false,
      alt_gr: false,
      ctrl: false,
      caps_lock: false,
      timestamp_ms: 115,
    });

    await Promise.all([p1, p2]);
    expect(order).toEqual(["KeyA", "KeyB"]);
    expect(useTypingStore.getState().statuses).toEqual(["matched", "matched", "pending"]);
    expect(useTypingStore.getState().graphemeIndex).toBe(2);
  });

  it("patches only updated indices on lightweight delta, handles unlock toast, and resets on rollover", () => {
    const store = useTypingStore.getState();
    store.applyDelta(makeDelta(1, 0));
    expect(useTypingStore.getState().statuses).toEqual(["matched", "pending", "pending"]);

    store.applyDelta({
      ...makeDelta(2, 1),
      newly_unlocked_key: {
        layout: "en_qwerty",
        key_id: "e",
        physical_code: "KeyE",
        requires_shift: false,
        requires_alt_gr: false,
        finger: "left_middle",
        unlock_order: 7,
        total_keys: 26,
      },
      updated_key_stats: [
        {
          layout: "en_qwerty",
          key_id: "e",
          physical_code: "KeyE",
          requires_shift: false,
          requires_alt_gr: false,
          unlock_order: 7,
          unlocked: true,
          hits: 0,
          misses: 0,
          ema_latency_ms: 0,
          confidence_score: 0,
          weakness_weight: 1,
          last_practiced_at: null,
          updated_at: "2026-11-25T16:00:00Z",
        },
      ],
    });
    expect(useTypingStore.getState().unlockedToast?.key_id).toBe("e");
    expect(useTypingStore.getState().keyStats).toHaveLength(1);

    store.applyDelta({
      ...makeDelta(0, 0),
      committed_updates: [],
      line_completed: true,
      next_line: {
        line_index: 1,
        words: ["ok"],
        graphemes: ["o", "k"],
      },
    });
    expect(useTypingStore.getState().lineIndex).toBe(1);
    expect(useTypingStore.getState().graphemes).toEqual(["o", "k"]);
    expect(useTypingStore.getState().statuses).toEqual(["pending", "pending"]);
  });

  it("auto-starts exam countdown on first keystroke and ticks down cleanly", () => {
    const store = useTypingStore.getState();
    store.applySnapshot(makeSnapshot(["t", "e", "s", "t"], "exam"));
    expect(useTypingStore.getState().examTimerRunning).toBe(false);

    store.applyDelta(makeDelta(1, 0));
    expect(useTypingStore.getState().examTimerRunning).toBe(true);

    store.tickExamTimer();
    expect(useTypingStore.getState().examRemainingSecs).toBe(59);
  });

  it("switches across Jungle Journey and triggers 5-star completion without mutating Adaptive stats", async () => {
    const prevWindow = (globalThis as { window?: unknown }).window;
    (globalThis as { window?: unknown }).window = { __TAURI_INTERNALS__: {} };

    try {
      vi.spyOn(commands, "getJungleRoadmap").mockResolvedValue({
        layout: "en_qwerty",
        total_earned_stars: 5,
        total_max_stars: 3500,
        completed_lessons: 1,
        total_lessons: 700,
        next_lesson_id: "u1-l2",
        units: [],
      });
      vi.spyOn(commands, "startJungleLesson").mockResolvedValue(
        makeSnapshot(["f", "j"], "jungle"),
      );
      vi.spyOn(commands, "completeJungleLesson").mockResolvedValue({
        lesson_id: "u1-l1",
        layout: "en_qwerty",
        kind: "key_intro",
        stars_earned: 4,
        best_stars: 4,
        is_new_best: true,
        passed: true,
        net_wpm: 22,
        accuracy: 96,
        target_wpm: 16,
        min_accuracy: 85,
        next_lesson_id: "u1-l2",
        next_lesson_unlocked: true,
      });

      const store = useTypingStore.getState();
      await store.setActiveTab("jungle");
      expect(useTypingStore.getState().jungleRoadmap?.total_lessons).toBe(700);

      await store.startJungleLesson("u1-l1");
      expect(useTypingStore.getState().practiceMode).toBe("jungle");
      expect(useTypingStore.getState().activeLessonMeta?.id).toBe("u1-l1");

      // Completing a Jungle lesson line (next_line: null) triggers 5-star grading.
      useTypingStore.getState().applyDelta({
        ...makeDelta(2, 1),
        line_completed: true,
        next_line: null,
        updated_key_stats: null,
      });

      await new Promise((r) => setTimeout(r, 10));
      expect(commands.completeJungleLesson).toHaveBeenCalledTimes(1);
      expect(useTypingStore.getState().lessonCompletion?.stars_earned).toBe(4);
      expect(useTypingStore.getState().keyStats).toEqual([]);
    } finally {
      vi.restoreAllMocks();
      if (prevWindow === undefined) {
        delete (globalThis as { window?: unknown }).window;
      } else {
        (globalThis as { window?: unknown }).window = prevWindow;
      }
    }
  });

  it("serializes Mini-Game keystrokes on FIFO queue via Rust IME resolver", async () => {
    vi.spyOn(commands, "resolveGameKeystroke").mockResolvedValue({
      locked_word: "किला",
      typed_preview: "कि",
      matched_graphemes: 1,
      total_graphemes: 2,
      pending_ime_preview: null,
      popped: false,
      is_error: false,
      sound_kind: "tap",
      metrics: {
        gross_wpm: 28,
        net_wpm: 28,
        kph: 8400,
        accuracy: 100,
        consistency: 100,
        elapsed_ms: 800,
        keystrokes: 2,
        errors: 0,
      },
    });

    try {
      const res = await useTypingStore.getState().resolveGameKeystroke(
        {
          code: "KeyD",
          shift: false,
          alt_gr: false,
          ctrl: false,
          caps_lock: false,
          timestamp_ms: 800,
        },
        ["किला"],
      );
      expect(res?.locked_word).toBe("किला");
      expect(useTypingStore.getState().metrics.net_wpm).toBe(28);
    } finally {
      vi.restoreAllMocks();
    }
  });
});
