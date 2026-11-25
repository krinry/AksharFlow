import { AlertTriangle, RotateCcw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";

function formatDuration(ms: number): string {
  const totalSecs = Math.floor(ms / 1000);
  const mins = Math.floor(totalSecs / 60);
  const secs = totalSecs % 60;
  return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
}

export function MetricsStrip() {
  const metrics = useTypingStore((s) => s.metrics);
  const lineIndex = useTypingStore((s) => s.lineIndex);
  const layout = useTypingStore((s) => s.layout);
  const capsLock = useTypingStore((s) => s.capsLock);
  const pendingImePreview = useTypingStore((s) => s.pendingImePreview);
  const uiLang = useTypingStore((s) => s.uiLang);
  const initSession = useTypingStore((s) => s.initSession);

  const t = getDictionary(uiLang);
  const isHindi = layout !== "en_qwerty";

  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div className="flex flex-wrap items-center gap-2 sm:gap-3">
        <MetricPill label={t.metrics.netWpm} value={String(Math.round(metrics.net_wpm ?? 0))} highlight />
        <MetricPill label={t.metrics.grossWpm} value={String(Math.round(metrics.gross_wpm ?? 0))} />
        <MetricPill label={t.metrics.accuracy} value={`${(metrics.accuracy ?? 100).toFixed(1)}%`} />
        <MetricPill label={t.metrics.consistency} value={`${Math.round(metrics.consistency ?? 100)}%`} />
        <MetricPill label={t.metrics.line} value={`#${lineIndex + 1}`} />
        <MetricPill label={t.metrics.time} value={formatDuration(metrics.elapsed_ms)} />
      </div>

      <div className="flex items-center gap-2">
        {capsLock && (
          <Badge variant="warning" title={isHindi ? t.typing.capsLockHindiHint : undefined}>
            <AlertTriangle className="h-3.5 w-3.5" />
            <span>{t.typing.capsLockOn}</span>
            {isHindi && (
              <span className="hidden opacity-80 sm:inline">· {t.typing.capsLockHindiHint}</span>
            )}
          </Badge>
        )}

        {pendingImePreview && (
          <Badge variant="accent">
            <span>{t.typing.pendingShortI}</span>
            <span className="font-[family-name:var(--font-devanagari)] text-sm font-bold">
              {pendingImePreview}
            </span>
          </Badge>
        )}

        <Button
          variant="secondary"
          size="sm"
          onClick={() => void initSession()}
          title={t.typing.restartShortcut}
        >
          <RotateCcw className="h-3.5 w-3.5" />
          <span>{t.typing.restart}</span>
        </Button>
      </div>
    </div>
  );
}

function MetricPill({
  label,
  value,
  highlight = false,
}: {
  label: string;
  value: string;
  highlight?: boolean;
}) {
  return (
    <div className="flex items-baseline gap-2 rounded-xl border border-border bg-card px-3 py-1.5 shadow-2xs">
      <span className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
        {label}
      </span>
      <span
        className={
          highlight
            ? "font-mono text-lg font-bold tabular-nums text-primary"
            : "font-mono text-sm font-semibold tabular-nums text-foreground"
        }
      >
        {value}
      </span>
    </div>
  );
}
