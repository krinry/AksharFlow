import { beforeEach, describe, expect, it, vi } from "vitest";
import { commands, type KeyEventDelta_Serialize, type SessionSnapshot_Serialize } from "@/lib/bindings";
import { resetKeyQueueForTest, useTypingStore } from "@/stores/useTypingStore";

function makeSnapshot(graphemes: string[]): SessionSnapshot_Serialize {
  return {
    session_id: "test-1",
    layout: "en_qwerty",
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
  };
}

describe("useTypingStore safeguards", () => {
  beforeEach(() => {
    resetKeyQueueForTest();
    useTypingStore.getState().applySnapshot(makeSnapshot(["a", "b", "c"]));
    useTypingStore.setState({ soundProfile: "off" });
  });

  it("serializes rapid keystrokes strictly in FIFO order", async () => {
    const order: string[] = [];
    vi.spyOn(commands, "processKeyEvent").mockImplementation(async (evt) => {
      // Delay first key longer than second key to prove FIFO serialization
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

  it("patches only updated indices on lightweight delta and resets on rollover", () => {
    const store = useTypingStore.getState();
    store.applyDelta(makeDelta(1, 0));
    expect(useTypingStore.getState().statuses).toEqual(["matched", "pending", "pending"]);

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
});
