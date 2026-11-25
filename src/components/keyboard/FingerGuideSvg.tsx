import type { FingerId } from "@/lib/bindings";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";

const FINGER_FILL: Record<FingerId, string> = {
  left_pinky: "#f43f5e",
  left_ring: "#f59e0b",
  left_middle: "#10b981",
  left_index: "#0ea5e9",
  thumb: "#f97316",
  right_index: "#6366f1",
  right_middle: "#10b981",
  right_ring: "#f59e0b",
  right_pinky: "#f43f5e",
};

interface FingerShape {
  id: FingerId;
  x: number;
  y: number;
  w: number;
  h: number;
  rotate?: number;
}

const LEFT_FINGERS: FingerShape[] = [
  { id: "left_pinky", x: 14, y: 40, w: 12, h: 42 },
  { id: "left_ring", x: 30, y: 24, w: 12, h: 56 },
  { id: "left_middle", x: 46, y: 16, w: 13, h: 64 },
  { id: "left_index", x: 63, y: 24, w: 12, h: 56 },
  { id: "thumb", x: 80, y: 66, w: 13, h: 32, rotate: 28 },
];

const RIGHT_FINGERS: FingerShape[] = [
  { id: "thumb", x: 115, y: 66, w: 13, h: 32, rotate: -28 },
  { id: "right_index", x: 133, y: 24, w: 12, h: 56 },
  { id: "right_middle", x: 149, y: 16, w: 13, h: 64 },
  { id: "right_ring", x: 166, y: 24, w: 12, h: 56 },
  { id: "right_pinky", x: 182, y: 40, w: 12, h: 42 },
];

export function FingerGuideSvg() {
  const nextKeyHint = useTypingStore((s) => s.nextKeyHint);
  const uiLang = useTypingStore((s) => s.uiLang);
  const t = getDictionary(uiLang);

  const activeFinger = nextKeyHint?.finger;
  const shiftNeedsLeftPinky = Boolean(
    nextKeyHint?.shift &&
      activeFinger &&
      activeFinger.startsWith("right_"),
  );
  const shiftNeedsRightPinky = Boolean(
    nextKeyHint?.shift &&
      activeFinger &&
      (activeFinger.startsWith("left_") || activeFinger === "thumb"),
  );

  const isHighlighted = (shape: FingerShape, hand: "left" | "right"): boolean => {
    if (shape.id === activeFinger) return true;
    if (hand === "left" && shiftNeedsLeftPinky && shape.id === "left_pinky") return true;
    if (hand === "right" && shiftNeedsRightPinky && shape.id === "right_pinky") return true;
    return false;
  };

  return (
    <aside className="flex flex-col items-center justify-center rounded-2xl border border-border bg-card/80 px-4 py-3 shadow-sm xl:w-64">
      <div className="mb-1.5 flex w-full items-center justify-between text-xs">
        <span className="font-semibold text-foreground">{t.fingers.guideTitle}</span>
        {activeFinger && (
          <span className="font-medium text-primary">{t.fingers[activeFinger]}</span>
        )}
      </div>

      <svg
        viewBox="0 0 210 135"
        className="h-32 w-full max-w-[220px]"
        aria-label={t.fingers.guideTitle}
      >
        {/* Palms */}
        <rect
          x="16"
          y="76"
          width="58"
          height="46"
          rx="14"
          className="fill-muted/70 stroke-border"
          strokeWidth="1.5"
        />
        <rect
          x="134"
          y="76"
          width="58"
          height="46"
          rx="14"
          className="fill-muted/70 stroke-border"
          strokeWidth="1.5"
        />

        {LEFT_FINGERS.map((f) => (
          <FingerNode
            key={`L-${f.id}`}
            shape={f}
            active={isHighlighted(f, "left")}
          />
        ))}
        {RIGHT_FINGERS.map((f) => (
          <FingerNode
            key={`R-${f.id}`}
            shape={f}
            active={isHighlighted(f, "right")}
          />
        ))}
      </svg>

      {nextKeyHint?.shift && (
        <span className="mt-1 rounded-full bg-amber-500/15 px-2 py-0.5 text-[11px] font-medium text-amber-600 dark:text-amber-300">
          {t.fingers.shiftPinky}
        </span>
      )}
    </aside>
  );
}

function FingerNode({ shape, active }: { shape: FingerShape; active: boolean }) {
  const fill = FINGER_FILL[shape.id];
  const cx = shape.x + shape.w / 2;
  const cy = shape.y + shape.h / 2;
  const transform = shape.rotate ? `rotate(${shape.rotate} ${cx} ${cy})` : undefined;

  return (
    <g transform={transform}>
      <rect
        x={shape.x}
        y={shape.y}
        width={shape.w}
        height={shape.h}
        rx={shape.w / 2}
        fill={active ? fill : "currentColor"}
        fillOpacity={active ? 0.92 : 0.12}
        stroke={fill}
        strokeWidth={active ? 2.2 : 1.2}
        className="transition-all duration-150"
      />
      {active && (
        <circle
          cx={cx}
          cy={shape.y + 6}
          r="3.2"
          fill="#ffffff"
        />
      )}
    </g>
  );
}
