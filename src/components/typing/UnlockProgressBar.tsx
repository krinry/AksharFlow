import { useMemo } from "react";
import { CheckCircle2, Lock, Target } from "lucide-react";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";
import { cn } from "@/lib/utils";

export function UnlockProgressBar() {
  const keyStats = useTypingStore((s) => s.keyStats);
  const settings = useTypingStore((s) => s.settings);
  const uiLang = useTypingStore((s) => s.uiLang);
  const layout = useTypingStore((s) => s.layout);

  const t = getDictionary(uiLang);
  const isHindi = layout !== "en_qwerty";
  const threshold = settings.unlock_threshold ?? 85;

  const { unlockedCount, topWeakIds } = useMemo(() => {
    const unlocked = keyStats.filter((k) => k.unlocked);
    const sorted = [...unlocked].sort((a, b) => {
      const aUnseen = a.hits + a.misses === 0 ? 1 : 0;
      const bUnseen = b.hits + b.misses === 0 ? 1 : 0;
      if (aUnseen !== bUnseen) return bUnseen - aUnseen;
      return (b.weakness_weight ?? 1) - (a.weakness_weight ?? 1);
    });
    return {
      unlockedCount: unlocked.length,
      topWeakIds: new Set(sorted.slice(0, 3).map((k) => k.key_id)),
    };
  }, [keyStats]);

  if (keyStats.length === 0) return null;

  return (
    <section className="w-full rounded-2xl border border-border/80 bg-card/75 px-4 py-3 shadow-2xs">
      <div className="mb-2 flex flex-wrap items-center justify-between gap-2 text-xs">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-foreground">
            {t.adaptive.unlockedKeys}:{" "}
            <span className="font-mono text-primary">
              {unlockedCount}/{keyStats.length}
            </span>
          </span>
        </div>
        <div className="flex items-center gap-3 text-[11px] text-muted-foreground">
          <span className="inline-flex items-center gap-1">
            <Target className="h-3 w-3 text-amber-500" />
            {t.adaptive.weakFocus}
          </span>
          <span className="inline-flex items-center gap-1">
            <CheckCircle2 className="h-3 w-3 text-emerald-500" />
            {Math.round(threshold)}% {t.adaptive.confidenceLabel}
          </span>
        </div>
      </div>

      <div className="flex flex-wrap gap-1.5">
        {keyStats.map((stat) => {
          const total = stat.hits + stat.misses;
          const conf = stat.confidence_score ?? 0;
          const acc = total > 0 ? ((stat.hits / total) * 100).toFixed(0) : "--";
          const lat = total > 0 ? `${Math.round(stat.ema_latency_ms ?? 0)}ms` : "--";
          const isMastered = stat.unlocked && total >= 12 && conf >= threshold;
          const isWeakFocus = stat.unlocked && topWeakIds.has(stat.key_id) && !isMastered;
          const progressPct = stat.unlocked
            ? Math.min(100, Math.max(0, (conf / Math.max(1, threshold)) * 100))
            : 0;

          return (
            <div
              key={stat.key_id}
              title={
                stat.unlocked
                  ? `${stat.key_id} · ${t.adaptive.confidenceLabel}: ${Math.round(conf)}% · ${t.metrics.accuracy}: ${acc}% · ${lat}`
                  : `${stat.key_id} · Locked`
              }
              className={cn(
                "relative flex h-8 min-w-[2rem] items-center justify-center overflow-hidden rounded-lg border px-1.5 text-xs font-semibold transition-colors",
                !stat.unlocked &&
                  "border-border/50 bg-muted/40 text-muted-foreground/45",
                stat.unlocked &&
                  !isMastered &&
                  !isWeakFocus &&
                  "border-border bg-background text-foreground",
                isWeakFocus &&
                  "border-amber-500/60 bg-amber-500/12 text-amber-700 dark:text-amber-300",
                isMastered &&
                  "border-emerald-500/45 bg-emerald-500/12 text-emerald-700 dark:text-emerald-300",
              )}
            >
              {!stat.unlocked ? (
                <span className="flex items-center gap-0.5">
                  <span className={cn(isHindi && "font-[family-name:var(--font-devanagari)] text-sm")}>
                    {stat.key_id}
                  </span>
                  <Lock className="h-2.5 w-2.5 opacity-60" />
                </span>
              ) : (
                <>
                  <span className={cn(isHindi && "font-[family-name:var(--font-devanagari)] text-sm")}>
                    {stat.key_id}
                  </span>
                  <span
                    style={{ width: `${progressPct}%` }}
                    className={cn(
                      "absolute bottom-0 left-0 h-[2.5px] transition-all",
                      isMastered ? "bg-emerald-500" : isWeakFocus ? "bg-amber-500" : "bg-primary",
                    )}
                  />
                </>
              )}
            </div>
          );
        })}
      </div>
    </section>
  );
}
