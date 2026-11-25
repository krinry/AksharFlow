import { useEffect } from "react";
import { ArrowRight, CheckCircle2, Map, RotateCcw, Sparkles, Star, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";
import { cn } from "@/lib/utils";

export function LessonCompleteModal() {
  const completion = useTypingStore((s) => s.lessonCompletion);
  const activeLessonMeta = useTypingStore((s) => s.activeLessonMeta);
  const uiLang = useTypingStore((s) => s.uiLang);
  const dismissLessonCompletion = useTypingStore((s) => s.dismissLessonCompletion);
  const exitActiveLessonToMap = useTypingStore((s) => s.exitActiveLessonToMap);
  const restartActiveLesson = useTypingStore((s) => s.restartActiveLesson);
  const startJungleLesson = useTypingStore((s) => s.startJungleLesson);

  const t = getDictionary(uiLang);

  useEffect(() => {
    if (!completion) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        dismissLessonCompletion();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [completion, dismissLessonCompletion]);

  if (!completion) return null;

  const canAdvance =
    Boolean(completion.next_lesson_id) &&
    (completion.next_lesson_unlocked || completion.best_stars >= 3);
  const lessonTitle = activeLessonMeta
    ? uiLang === "hi"
      ? activeLessonMeta.title_hi
      : activeLessonMeta.title_en
    : completion.lesson_id;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="lesson-complete-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/55 p-4 backdrop-blur-xs"
    >
      <div className="w-full max-w-md rounded-2xl border border-border bg-card p-6 shadow-xl">
        <div className="mb-4 flex items-start justify-between gap-3">
          <div>
            <h2 id="lesson-complete-title" className="text-lg font-bold text-foreground">
              {t.jungle.completeTitle}
            </h2>
            <p className="mt-0.5 text-xs text-muted-foreground">
              {lessonTitle} · {t.layouts[completion.layout]}
            </p>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            aria-label={t.settings.close}
            onClick={dismissLessonCompletion}
          >
            <X className="h-4 w-4" aria-hidden="true" />
          </Button>
        </div>

        <div
          className="mb-5 flex flex-col items-center rounded-xl border border-border/80 bg-muted/40 py-5"
          aria-label={`${completion.stars_earned} of 5 ${t.jungle.starsLabel}`}
        >
          <div className="flex items-center gap-2">
            {[1, 2, 3, 4, 5].map((n) => {
              const filled = n <= completion.stars_earned;
              return (
                <Star
                  key={n}
                  aria-hidden="true"
                  className={cn(
                    "h-8 w-8 transition-transform",
                    filled
                      ? "fill-amber-400 text-amber-500 scale-105"
                      : "text-muted-foreground/35",
                  )}
                />
              );
            })}
          </div>

          <div className="mt-3 flex flex-wrap items-center justify-center gap-2 text-xs font-semibold">
            {completion.passed ? (
              <span className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-3 py-1 text-emerald-700 dark:text-emerald-300">
                <CheckCircle2 className="h-3.5 w-3.5" aria-hidden="true" />
                {t.jungle.passedBadge}
              </span>
            ) : (
              <span className="rounded-full bg-amber-500/15 px-3 py-1 text-center text-amber-800 dark:text-amber-200">
                {t.jungle.needThreeStars}
              </span>
            )}

            {completion.is_new_best && (
              <span className="inline-flex items-center gap-1 rounded-full bg-primary/15 px-2.5 py-1 text-primary">
                <Sparkles className="h-3 w-3" aria-hidden="true" />
                {t.jungle.newBest}
              </span>
            )}
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3 text-sm">
          <div className="rounded-xl border border-border bg-background/70 px-3.5 py-3">
            <div className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
              {t.metrics.netWpm}
            </div>
            <div className="mt-1 flex items-baseline justify-between">
              <span className="font-mono text-2xl font-bold tabular-nums text-primary">
                {Math.round(completion.net_wpm ?? 0)}
              </span>
              <span className="font-mono text-xs text-muted-foreground">
                {t.jungle.targetLabel} {Math.round(completion.target_wpm ?? 0)}
              </span>
            </div>
          </div>

          <div className="rounded-xl border border-border bg-background/70 px-3.5 py-3">
            <div className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
              {t.metrics.accuracy}
            </div>
            <div className="mt-1 flex items-baseline justify-between">
              <span className="font-mono text-2xl font-bold tabular-nums text-emerald-600 dark:text-emerald-400">
                {(completion.accuracy ?? 0).toFixed(1)}%
              </span>
              <span className="font-mono text-xs text-muted-foreground">
                {t.jungle.targetLabel} {Math.round(completion.min_accuracy ?? 0)}%
              </span>
            </div>
          </div>
        </div>

        <div className="mt-5 flex flex-wrap items-center justify-between gap-2 border-t border-border pt-4">
          <Button type="button" variant="outline" size="sm" onClick={exitActiveLessonToMap}>
            <Map className="h-3.5 w-3.5" aria-hidden="true" />
            {t.jungle.backToMap}
          </Button>

          <div className="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              variant="secondary"
              size="sm"
              onClick={() => void restartActiveLesson()}
            >
              <RotateCcw className="h-3.5 w-3.5" aria-hidden="true" />
              {t.jungle.retry}
            </Button>

            {canAdvance && completion.next_lesson_id && (
              <Button
                type="button"
                variant="default"
                size="sm"
                onClick={() => void startJungleLesson(completion.next_lesson_id!)}
              >
                {t.jungle.nextLesson}
                <ArrowRight className="h-3.5 w-3.5" aria-hidden="true" />
              </Button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
