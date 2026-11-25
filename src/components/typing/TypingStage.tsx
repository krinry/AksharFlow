import { useEffect, useMemo, useRef, useState } from "react";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";
import { cn } from "@/lib/utils";

interface WordGroup {
  startIndex: number;
  items: { glyph: string; index: number }[];
}

export function TypingStage() {
  const stageRef = useRef<HTMLDivElement>(null);
  const tabArmedRef = useRef(false);
  const [focused, setFocused] = useState(true);

  const layout = useTypingStore((s) => s.layout);
  const practiceMode = useTypingStore((s) => s.practiceMode);
  const uiLang = useTypingStore((s) => s.uiLang);
  const graphemes = useTypingStore((s) => s.graphemes);
  const statuses = useTypingStore((s) => s.statuses);
  const graphemeIndex = useTypingStore((s) => s.graphemeIndex);
  const activeTypedPreview = useTypingStore((s) => s.activeTypedPreview);
  const activeIsPartial = useTypingStore((s) => s.activeIsPartial);
  const activeHasError = useTypingStore((s) => s.activeHasError);
  const pendingImePreview = useTypingStore((s) => s.pendingImePreview);
  const isReady = useTypingStore((s) => s.isReady);
  const examSummary = useTypingStore((s) => s.examSummary);
  const lessonCompletion = useTypingStore((s) => s.lessonCompletion);
  const errorMessage = useTypingStore((s) => s.errorMessage);
  const enqueueKeyInput = useTypingStore((s) => s.enqueueKeyInput);
  const setModifiers = useTypingStore((s) => s.setModifiers);
  const restartActiveLesson = useTypingStore((s) => s.restartActiveLesson);

  const t = getDictionary(uiLang);
  const isHindi = layout !== "en_qwerty";

  useEffect(() => {
    stageRef.current?.focus();
  }, [layout, isReady]);

  // Group graphemes by spaces so words wrap cleanly as atomic units without breaking mid-word.
  const wordGroups = useMemo<WordGroup[]>(() => {
    const groups: WordGroup[] = [];
    let current: WordGroup = { startIndex: 0, items: [] };
    graphemes.forEach((glyph, index) => {
      if (glyph === " ") {
        if (current.items.length > 0) {
          groups.push(current);
        }
        groups.push({ startIndex: index, items: [{ glyph, index }] });
        current = { startIndex: index + 1, items: [] };
      } else {
        current.items.push({ glyph, index });
      }
    });
    if (current.items.length > 0) {
      groups.push(current);
    }
    return groups;
  }, [graphemes]);

  const syncModifierFlags = (e: React.KeyboardEvent<HTMLDivElement>) => {
    setModifiers({
      capsLock: e.getModifierState("CapsLock"),
      shiftHeld: e.shiftKey,
      altGrHeld: e.getModifierState("AltGraph"),
    });
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    const timestampMs = performance.now();
    syncModifierFlags(e);

    // Quick restart shortcut: Tab followed by Enter
    if (e.code === "Tab") {
      e.preventDefault();
      tabArmedRef.current = true;
      return;
    }
    if (e.code === "Enter" && tabArmedRef.current) {
      e.preventDefault();
      tabArmedRef.current = false;
      void restartActiveLesson();
      return;
    }
    tabArmedRef.current = false;

    // Ignore pure modifier keys or browser/OS shortcuts (e.g. Ctrl+R, Alt+Tab), keeping Ctrl+Backspace
    if (
      e.key === "Shift" ||
      e.key === "Control" ||
      e.key === "Alt" ||
      e.key === "AltGraph" ||
      e.key === "CapsLock" ||
      e.key === "Meta" ||
      e.metaKey ||
      (e.ctrlKey && e.code !== "Backspace")
    ) {
      return;
    }

    if (!isReady || examSummary || lessonCompletion || practiceMode === "mini_game") return;
    e.preventDefault();

    void enqueueKeyInput({
      code: e.code,
      shift: e.shiftKey,
      alt_gr: e.getModifierState("AltGraph"),
      ctrl: e.ctrlKey,
      caps_lock: e.getModifierState("CapsLock"),
      timestamp_ms: timestampMs,
    });
  };

  const handleKeyUp = (e: React.KeyboardEvent<HTMLDivElement>) => {
    syncModifierFlags(e);
  };

  return (
    <div
      ref={stageRef}
      tabIndex={0}
      role="textbox"
      aria-label="Typing Stage"
      onKeyDown={handleKeyDown}
      onKeyUp={handleKeyUp}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      onClick={() => stageRef.current?.focus()}
      className={cn(
        "relative min-h-[190px] w-full rounded-2xl border bg-card px-6 py-7 shadow-sm transition-all cursor-text focus-visible:ring-2 focus-visible:ring-primary",
        focused ? "border-primary/50 ring-2 ring-primary/15" : "border-border",
      )}
    >
      {errorMessage && (
        <div className="mb-3 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive">
          {errorMessage}
        </div>
      )}

      <div
        className={cn(
          "flex flex-wrap items-baseline select-none",
          isHindi
            ? "font-[family-name:var(--font-devanagari)] text-[1.9rem] leading-[2.25] tracking-normal"
            : "font-mono text-[1.6rem] leading-[1.95] tracking-wide",
        )}
      >
        {wordGroups.map((group) => {
          const isSpace = group.items.length === 1 && group.items[0].glyph === " ";
          if (isSpace) {
            const item = group.items[0];
            const status = statuses[item.index] ?? "pending";
            const isActive = item.index === graphemeIndex;
            return (
              <span
                key={item.index}
                className={cn(
                  "relative inline-block w-[0.55em] text-center rounded-sm transition-colors",
                  isActive && "bg-primary/20",
                  status === "error" && "bg-destructive/25 underline decoration-destructive decoration-2",
                )}
              >
                {isActive && (
                  <span className="animate-caret absolute left-0 top-[18%] h-[68%] w-[2.5px] rounded-full bg-primary" />
                )}
                &nbsp;
              </span>
            );
          }

          return (
            <span key={group.startIndex} className="inline-flex items-baseline whitespace-nowrap">
              {group.items.map(({ glyph, index }) => {
                const status = statuses[index] ?? "pending";
                const isActive = index === graphemeIndex;
                const previewText = isActive
                  ? pendingImePreview && activeTypedPreview
                    ? `${activeTypedPreview} + ${pendingImePreview}`
                    : pendingImePreview || activeTypedPreview
                  : "";

                return (
                  <span
                    key={index}
                    className={cn(
                      "relative inline-block rounded-[3px] px-[0.5px] transition-colors",
                      status === "pending" && !isActive && "text-muted-foreground/75",
                      status === "matched" && "text-emerald-600 dark:text-emerald-400",
                      status === "error" &&
                        "text-destructive underline decoration-destructive decoration-2 underline-offset-4",
                      isActive && !activeHasError && "text-foreground bg-primary/12",
                      isActive &&
                        activeIsPartial &&
                        !activeHasError &&
                        "text-amber-600 dark:text-amber-300 underline decoration-amber-500/80 decoration-2 underline-offset-4",
                      isActive &&
                        activeHasError &&
                        "text-destructive bg-destructive/15 underline decoration-destructive decoration-2 underline-offset-4",
                    )}
                  >
                    {isActive && (
                      <span className="animate-caret absolute -left-[1px] top-[16%] h-[68%] w-[2.5px] rounded-full bg-primary" />
                    )}
                    {isActive && previewText && (
                      <span className="pointer-events-none absolute -top-6 left-1/2 -translate-x-1/2 whitespace-nowrap rounded bg-popover/95 px-1.5 py-0.5 text-xs font-semibold text-primary shadow ring-1 ring-primary/35">
                        {previewText}
                      </span>
                    )}
                    {glyph}
                  </span>
                );
              })}
            </span>
          );
        })}
      </div>

      {!focused && (
        <div className="absolute inset-0 flex items-center justify-center rounded-2xl bg-background/75 backdrop-blur-[2px]">
          <div className="rounded-xl border border-border bg-card px-4 py-2 text-sm font-medium text-foreground shadow-sm">
            {t.typing.focusPrompt}
          </div>
        </div>
      )}
    </div>
  );
}
