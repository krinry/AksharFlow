import { useMemo } from "react";
import { Activity, Award, Compass, Crosshair, Flame, History, Star, Zap } from "lucide-react";
import { Button } from "@/components/ui/button";
import { WpmTrendChart } from "@/components/analytics/WpmTrendChart";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";
import { cn } from "@/lib/utils";

export function AnalyticsDashboard() {
  const analytics = useTypingStore((s) => s.analytics);
  const analyticsLoading = useTypingStore((s) => s.analyticsLoading);
  const layout = useTypingStore((s) => s.layout);
  const uiLang = useTypingStore((s) => s.uiLang);
  const startWeakKeyDrill = useTypingStore((s) => s.startWeakKeyDrill);
  const setActiveTab = useTypingStore((s) => s.setActiveTab);

  const t = getDictionary(uiLang);
  const isHindi = layout !== "en_qwerty";

  const top5WeakKeys = useMemo(() => {
    const stats = analytics?.key_stats ?? [];
    const unlocked = stats.filter((k) => k.unlocked);
    const practiced = unlocked.filter((k) => k.hits + k.misses > 0);
    const source = practiced.length > 0 ? practiced : unlocked;
    return [...source]
      .sort((a, b) => (b.weakness_weight ?? 0) - (a.weakness_weight ?? 0))
      .slice(0, 5);
  }, [analytics]);

  const weakKeyIds = useMemo(() => top5WeakKeys.map((k) => k.key_id), [top5WeakKeys]);

  const unlockedPct = analytics && analytics.total_keys > 0
    ? `${analytics.unlocked_count}/${analytics.total_keys}`
    : "--";

  const earnedStars = analytics?.jungle_earned_stars ?? 0;
  const maxStars = analytics?.jungle_max_stars ?? 3500;
  const clearedLessons = analytics?.jungle_completed_lessons ?? 0;
  const totalLessons = analytics?.jungle_total_lessons ?? 700;
  const junglePct = totalLessons > 0 ? Math.round((clearedLessons / totalLessons) * 100) : 0;

  if (analyticsLoading && !analytics) {
    return (
      <div className="flex min-h-[260px] items-center justify-center rounded-2xl border border-border bg-card p-8 text-sm text-muted-foreground">
        Loading telemetry...
      </div>
    );
  }

  return (
    <div className="flex w-full flex-col gap-5">
      {/* Jungle Journey Mastery Summary */}
      <section className="flex flex-col justify-between gap-4 rounded-2xl border border-border bg-card p-4 shadow-xs sm:flex-row sm:items-center">
        <div className="space-y-1.5">
          <div className="flex items-center gap-2">
            <Compass className="h-4 w-4 text-primary" aria-hidden="true" />
            <h2 className="text-sm font-bold text-foreground">
              {t.analytics.jungleProgressTitle}
            </h2>
            <span className="rounded-md bg-muted px-2 py-0.5 text-[11px] font-medium text-muted-foreground">
              {t.layouts[layout]}
            </span>
          </div>
          <div className="flex flex-wrap items-center gap-4 text-xs">
            <span className="inline-flex items-center gap-1.5 font-mono font-semibold text-foreground">
              <Star className="h-3.5 w-3.5 fill-amber-400 text-amber-500" aria-hidden="true" />
              {t.analytics.starsEarned}: {earnedStars.toLocaleString()} / {maxStars.toLocaleString()}
            </span>
            <span className="font-mono text-muted-foreground">
              {t.analytics.lessonsCompleted}: {clearedLessons} / {totalLessons} ({junglePct}%)
            </span>
          </div>
          <div className="h-2 w-full max-w-md overflow-hidden rounded-full bg-muted">
            <div
              className="h-full rounded-full bg-primary transition-all"
              style={{ width: `${Math.max(earnedStars > 0 ? 2 : 0, junglePct)}%` }}
            />
          </div>
        </div>

        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => void setActiveTab("jungle")}
        >
          <Compass className="h-3.5 w-3.5" aria-hidden="true" />
          {t.nav.jungle}
        </Button>
      </section>

      {/* KPI Cards */}
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-5">
        <KpiCard
          icon={<Zap className="h-4 w-4 text-primary" aria-hidden="true" />}
          label={t.analytics.avgNetWpm}
          value={String(analytics?.avg_net_wpm ?? 0)}
        />
        <KpiCard
          icon={<Award className="h-4 w-4 text-amber-700 dark:text-amber-400" aria-hidden="true" />}
          label={t.analytics.bestNetWpm}
          value={String(analytics?.best_net_wpm ?? 0)}
        />
        <KpiCard
          icon={<Activity className="h-4 w-4 text-emerald-700 dark:text-emerald-400" aria-hidden="true" />}
          label={t.analytics.avgAccuracy}
          value={`${(analytics?.avg_accuracy ?? 100).toFixed(1)}%`}
        />
        <KpiCard
          icon={<History className="h-4 w-4 text-sky-700 dark:text-sky-400" aria-hidden="true" />}
          label={t.analytics.totalSessions}
          value={String(analytics?.total_sessions ?? 0)}
        />
        <KpiCard
          icon={<Flame className="h-4 w-4 text-rose-700 dark:text-rose-400" aria-hidden="true" />}
          label={t.adaptive.unlockedKeys}
          value={unlockedPct}
        />
      </div>

      {/* Main Grid: WPM Trend Chart + Top 5 Weak Keys */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
        <section className="rounded-2xl border border-border bg-card p-4 shadow-sm lg:col-span-2">
          <h2 className="mb-3 text-sm font-bold">{t.analytics.trendTitle}</h2>
          <WpmTrendChart sessions={analytics?.recent_sessions ?? []} />
        </section>

        <section className="flex flex-col justify-between rounded-2xl border border-border bg-card p-4 shadow-sm">
          <div>
            <div className="mb-3 flex items-center justify-between">
              <h2 className="flex items-center gap-1.5 text-sm font-bold">
                <Crosshair className="h-4 w-4 text-amber-500" />
                {t.analytics.weakKeysTitle}
              </h2>
            </div>

            {top5WeakKeys.length === 0 ? (
              <p className="text-xs text-muted-foreground">{t.analytics.noWeakData}</p>
            ) : (
              <div className="space-y-2.5">
                {top5WeakKeys.map((stat) => {
                  const total = stat.hits + stat.misses;
                  const acc = total > 0 ? ((stat.hits / total) * 100).toFixed(0) : "--";
                  const conf = Math.round(stat.confidence_score ?? 0);
                  const weaknessPct = Math.round((stat.weakness_weight ?? 0) * 100);
                  return (
                    <div
                      key={stat.key_id}
                      className="rounded-xl border border-border/70 bg-background/70 px-3 py-2"
                    >
                      <div className="flex items-center justify-between text-xs">
                        <div className="flex items-center gap-2">
                          <span
                            className={cn(
                              "flex h-7 min-w-[1.75rem] items-center justify-center rounded-md border border-border bg-card px-1.5 font-bold",
                              isHindi
                                ? "font-[family-name:var(--font-devanagari)] text-sm"
                                : "font-mono text-xs",
                            )}
                          >
                            {stat.key_id}
                          </span>
                          <span className="text-muted-foreground">
                            {acc}% · {Math.round(stat.ema_latency_ms ?? 0)}ms
                          </span>
                        </div>
                        <span className="font-mono font-semibold text-amber-700 dark:text-amber-400">
                          {conf}% ({total} {t.analytics.attempts})
                        </span>
                      </div>
                      <div className="mt-1.5 h-1.5 w-full overflow-hidden rounded-full bg-muted">
                        <div
                          className="h-full rounded-full bg-gradient-to-r from-amber-500 to-rose-500"
                          style={{ width: `${Math.max(8, weaknessPct)}%` }}
                        />
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          <Button
            className="mt-4 w-full"
            disabled={weakKeyIds.length === 0}
            onClick={() => void startWeakKeyDrill(weakKeyIds)}
          >
            <Crosshair className="h-4 w-4" />
            {t.analytics.practiceWeakKeys}
          </Button>
        </section>
      </div>

      {/* Session History Table */}
      <section className="rounded-2xl border border-border bg-card p-4 shadow-sm">
        <h2 className="mb-3 text-sm font-bold">{t.analytics.historyTitle}</h2>
        {!analytics || analytics.recent_sessions.length === 0 ? (
          <p className="text-xs text-muted-foreground">{t.analytics.noData}</p>
        ) : (
          <div className="max-h-60 overflow-auto">
            <table className="w-full text-left text-xs">
              <thead className="border-b border-border text-[11px] uppercase text-muted-foreground">
                <tr>
                  <th className="py-2 pr-3">{t.analytics.mode}</th>
                  <th className="py-2 pr-3">{t.metrics.netWpm}</th>
                  <th className="py-2 pr-3">{t.metrics.grossWpm}</th>
                  <th className="py-2 pr-3">{t.metrics.kph}</th>
                  <th className="py-2 pr-3">{t.metrics.accuracy}</th>
                  <th className="py-2 pr-3">{t.metrics.consistency}</th>
                  <th className="py-2 pr-3">{t.metrics.errors}</th>
                  <th className="py-2">{t.analytics.date}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border/60">
                {analytics.recent_sessions.slice(0, 20).map((s) => (
                  <tr key={s.id} className="hover:bg-muted/40">
                    <td className="py-2 pr-3 font-medium capitalize">{s.mode.replace("_", " ")}</td>
                    <td className="py-2 pr-3 font-mono font-bold text-primary">
                      {Math.round(s.net_wpm ?? 0)}
                    </td>
                    <td className="py-2 pr-3 font-mono">{Math.round(s.gross_wpm ?? 0)}</td>
                    <td className="py-2 pr-3 font-mono">{Math.round(s.kph ?? 0)}</td>
                    <td className="py-2 pr-3 font-mono text-emerald-700 dark:text-emerald-400">
                      {(s.accuracy ?? 100).toFixed(1)}%
                    </td>
                    <td className="py-2 pr-3 font-mono">{Math.round(s.consistency ?? 100)}%</td>
                    <td className="py-2 pr-3 font-mono text-rose-700 dark:text-rose-400">
                      {s.errors}
                    </td>
                    <td className="py-2 text-muted-foreground">
                      {new Date(s.created_at).toLocaleString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}

function KpiCard({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="rounded-2xl border border-border bg-card p-3.5 shadow-2xs">
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <span>{label}</span>
        {icon}
      </div>
      <div className="mt-1 font-mono text-2xl font-bold tabular-nums">{value}</div>
    </div>
  );
}
