import { useMemo } from "react";
import { Flame } from "lucide-react";
import { getDictionary } from "@/i18n/useTranslation";
import type { FingerId, KeycapDescriptor_Serialize } from "@/lib/bindings";
import { useTypingStore } from "@/stores/useTypingStore";
import { cn } from "@/lib/utils";

const FINGER_BORDER: Record<FingerId, string> = {
  left_pinky: "border-b-rose-500/70",
  left_ring: "border-b-amber-500/70",
  left_middle: "border-b-emerald-500/70",
  left_index: "border-b-sky-500/70",
  thumb: "border-b-primary/80",
  right_index: "border-b-indigo-500/70",
  right_middle: "border-b-emerald-500/70",
  right_ring: "border-b-amber-500/70",
  right_pinky: "border-b-rose-500/70",
};

export function VirtualKeyboard() {
  const keycaps = useTypingStore((s) => s.keycaps);
  const keyStats = useTypingStore((s) => s.keyStats);
  const layout = useTypingStore((s) => s.layout);
  const uiLang = useTypingStore((s) => s.uiLang);
  const nextKeyHint = useTypingStore((s) => s.nextKeyHint);
  const shiftHeld = useTypingStore((s) => s.shiftHeld);
  const altGrHeld = useTypingStore((s) => s.altGrHeld);
  const touchShift = useTypingStore((s) => s.touchShift);
  const touchAltGr = useTypingStore((s) => s.touchAltGr);
  const capsLock = useTypingStore((s) => s.capsLock);
  const showHeatmap = useTypingStore((s) => s.showHeatmap);
  const isReady = useTypingStore((s) => s.isReady);
  const toggleHeatmap = useTypingStore((s) => s.toggleHeatmap);
  const toggleTouchShift = useTypingStore((s) => s.toggleTouchShift);
  const toggleTouchAltGr = useTypingStore((s) => s.toggleTouchAltGr);
  const enqueueKeyInput = useTypingStore((s) => s.enqueueKeyInput);

  const t = getDictionary(uiLang);
  const isHindi = layout !== "en_qwerty";
  const effectiveShift = shiftHeld || touchShift;
  const effectiveAltGr = altGrHeld || touchAltGr;

  const rows = useMemo(() => {
    const grouped: KeycapDescriptor_Serialize[][] = [[], [], [], [], []];
    for (const cap of keycaps) {
      if (cap.row >= 0 && cap.row < grouped.length) {
        grouped[cap.row].push(cap);
      }
    }
    return grouped;
  }, [keycaps]);

  // Map physical key code + modifier state -> confidence color class for Red-to-Green Heatmap
  const heatmapByCode = useMemo(() => {
    const map = new Map<string, { conf: number; practiced: boolean; unlocked: boolean }>();
    for (const stat of keyStats) {
      if (stat.requires_shift !== effectiveShift || stat.requires_alt_gr !== effectiveAltGr) {
        continue;
      }
      const practiced = stat.hits + stat.misses > 0;
      map.set(stat.physical_code, {
        conf: stat.confidence_score ?? 0,
        practiced,
        unlocked: stat.unlocked,
      });
    }
    return map;
  }, [keyStats, effectiveShift, effectiveAltGr]);

  const dispatchVirtualKey = (code: string) => {
    if (!isReady) return;
    const timestampMs = performance.now();
    void enqueueKeyInput({
      code,
      shift: effectiveShift,
      alt_gr: effectiveAltGr,
      ctrl: false,
      caps_lock: capsLock,
      timestamp_ms: timestampMs,
    });
    if (touchShift) toggleTouchShift();
    if (touchAltGr) toggleTouchAltGr();
  };

  return (
    <section className="w-full rounded-2xl border border-border bg-card/80 p-3.5 shadow-sm">
      <div className="mb-2.5 flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
        <div className="flex items-center gap-2">
          {nextKeyHint ? (
            <>
              <span className="font-medium text-foreground">{t.typing.nextKey}:</span>
              {nextKeyHint.shift && (
                <span className="rounded bg-amber-500/20 px-1.5 py-0.5 font-semibold text-amber-600 dark:text-amber-300">
                  {t.typing.withShift} +
                </span>
              )}
              {nextKeyHint.alt_gr && (
                <span className="rounded bg-sky-500/20 px-1.5 py-0.5 font-semibold text-sky-600 dark:text-sky-300">
                  {t.typing.withAltGr} +
                </span>
              )}
              <kbd className="rounded-md border border-primary/50 bg-primary/15 px-2 py-0.5 font-mono text-xs font-bold text-primary">
                {nextKeyHint.expected_label}
              </kbd>
              <span>· {t.fingers[nextKeyHint.finger]}</span>
            </>
          ) : (
            <span>{t.typing.touchModeHint}</span>
          )}
        </div>

        <div className="flex items-center gap-3">
          <span className="hidden text-[11px] sm:inline">{t.typing.touchModeHint}</span>
          <button
            type="button"
            onClick={toggleHeatmap}
            className={cn(
              "inline-flex items-center gap-1 rounded-lg border px-2 py-0.5 text-[11px] font-medium transition-colors cursor-pointer",
              showHeatmap
                ? "border-primary/45 bg-primary/15 text-primary"
                : "border-border bg-background text-muted-foreground hover:text-foreground",
            )}
          >
            <Flame className="h-3 w-3" />
            {t.adaptive.heatmap}
          </button>
        </div>
      </div>

      <div className="mx-auto flex max-w-[820px] flex-col gap-1.5 select-none">
        {rows.slice(0, 4).map((rowCaps, rowIdx) => (
          <div key={rowIdx} className="flex justify-center gap-1.5">
            {rowIdx === 3 && (
              <ModifierKeyButton
                label="Shift"
                active={effectiveShift}
                highlighted={Boolean(nextKeyHint?.shift)}
                onPress={toggleTouchShift}
              />
            )}
            {rowCaps.map((cap) => {
              const isTarget = nextKeyHint?.code === cap.code;
              const legend = effectiveAltGr && cap.alt_gr_label
                ? cap.alt_gr_label
                : effectiveShift
                  ? cap.shift_label
                  : cap.normal_label;
              const subLegend =
                !effectiveShift && !effectiveAltGr && cap.shift_label !== cap.normal_label
                  ? cap.shift_label
                  : null;
              const heat = showHeatmap ? heatmapByCode.get(cap.code) : undefined;
              const heatClass =
                heat && heat.unlocked && heat.practiced
                  ? heat.conf >= 85
                    ? "bg-emerald-500/14 border-emerald-500/40"
                    : heat.conf >= 60
                      ? "bg-amber-500/14 border-amber-500/40"
                      : "bg-rose-500/16 border-rose-500/45"
                  : "";

              return (
                <button
                  key={cap.code}
                  type="button"
                  onPointerDown={(e) => {
                    e.preventDefault();
                    dispatchVirtualKey(cap.code);
                  }}
                  style={{ flexGrow: cap.width_u ?? 1, flexBasis: 0 }}
                  className={cn(
                    "relative flex h-11 min-w-[2rem] flex-col items-center justify-center rounded-lg border border-b-[3px] px-1 transition-all active:translate-y-[1px] cursor-pointer",
                    FINGER_BORDER[cap.finger],
                    isTarget
                      ? "border-primary bg-primary/20 text-primary shadow-xs ring-2 ring-primary/40"
                      : cn("border-border bg-background/85 text-foreground hover:bg-muted", heatClass),
                  )}
                >
                  {subLegend && cap.code !== "Backspace" && (
                    <span className="absolute right-1 top-0.5 text-[9px] leading-none text-muted-foreground">
                      {subLegend}
                    </span>
                  )}
                  <span
                    className={cn(
                      "leading-none font-semibold",
                      isHindi && cap.code !== "Backspace"
                        ? "font-[family-name:var(--font-devanagari)] text-base"
                        : "font-mono text-sm",
                    )}
                  >
                    {legend || " "}
                  </span>
                </button>
              );
            })}
            {rowIdx === 3 && (
              <ModifierKeyButton
                label="Shift"
                active={effectiveShift}
                highlighted={Boolean(nextKeyHint?.shift)}
                onPress={toggleTouchShift}
              />
            )}
          </div>
        ))}

        <div className="flex justify-center gap-1.5">
          {rows[4]?.[0] && (
            <button
              key="Space"
              type="button"
              onPointerDown={(e) => {
                e.preventDefault();
                dispatchVirtualKey("Space");
              }}
              className={cn(
                "flex h-10 w-64 max-w-[55%] items-center justify-center rounded-lg border border-b-[3px] text-xs font-semibold transition-all active:translate-y-[1px] cursor-pointer",
                FINGER_BORDER.thumb,
                nextKeyHint?.code === "Space"
                  ? "border-primary bg-primary/20 text-primary ring-2 ring-primary/40"
                  : "border-border bg-background/85 text-muted-foreground hover:bg-muted",
              )}
            >
              Space
            </button>
          )}
          {isHindi && (
            <ModifierKeyButton
              label="AltGr"
              active={effectiveAltGr}
              highlighted={Boolean(nextKeyHint?.alt_gr)}
              onPress={toggleTouchAltGr}
            />
          )}
        </div>
      </div>
    </section>
  );
}

function ModifierKeyButton({
  label,
  active,
  highlighted,
  onPress,
}: {
  label: string;
  active: boolean;
  highlighted: boolean;
  onPress: () => void;
}) {
  return (
    <button
      type="button"
      onPointerDown={(e) => {
        e.preventDefault();
        onPress();
      }}
      className={cn(
        "flex h-11 min-w-[4.5rem] items-center justify-center rounded-lg border border-b-[3px] px-2.5 text-xs font-semibold transition-all cursor-pointer",
        active
          ? "border-primary bg-primary text-primary-foreground"
          : highlighted
            ? "border-amber-500 bg-amber-500/20 text-amber-600 dark:text-amber-300 ring-2 ring-amber-500/40"
            : "border-border bg-muted/70 text-muted-foreground hover:bg-muted hover:text-foreground",
      )}
    >
      {label}
    </button>
  );
}
