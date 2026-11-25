import { useEffect, useMemo, useState } from "react";
import {
  AlignLeft,
  Compass,
  Crown,
  Gamepad2,
  KeyRound,
  Lock,
  Play,
  RefreshCw,
  Repeat,
  Star,
  Type,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { getDictionary } from "@/i18n/useTranslation";
import type { CurriculumStage, LessonKind, LessonNodeSummary, UnitSummary } from "@/lib/bindings";
import { useTypingStore } from "@/stores/useTypingStore";
import { cn } from "@/lib/utils";

const STAGES: ("all" | CurriculumStage)[] = [
  "all",
  "home_row",
  "top_row_and_matras",
  "bottom_row_and_conjuncts",
  "shift_numbers_and_symbols",
  "fluency_and_stories",
];

function lessonKindIcon(kind: LessonKind) {
  switch (kind) {
    case "key_intro":
      return KeyRound;
    case "drill":
      return Repeat;
    case "words":
      return Type;
    case "sentences":
      return AlignLeft;
    case "review_challenge":
      return Crown;
    case "mini_game":
      return Gamepad2;
  }
}

export function JungleMap() {
  const layout = useTypingStore((s) => s.layout);
  const uiLang = useTypingStore((s) => s.uiLang);
  const roadmap = useTypingStore((s) => s.jungleRoadmap);
  const roadmapLoading = useTypingStore((s) => s.roadmapLoading);
  const roadmapError = useTypingStore((s) => s.roadmapError);
  const loadJungleRoadmap = useTypingStore((s) => s.loadJungleRoadmap);
  const startJungleLesson = useTypingStore((s) => s.startJungleLesson);

  const [selectedStage, setSelectedStage] = useState<"all" | CurriculumStage>("all");
  const [selectedUnitNum, setSelectedUnitNum] = useState<number>(1);

  const t = getDictionary(uiLang);
  const isHindi = layout !== "en_qwerty";

  // Jump initial unit focus to the unit containing the next playable lesson when roadmap loads.
  useEffect(() => {
    if (!roadmap) return;
    const nextUnit = roadmap.units.find((u) =>
      u.lessons.some((l) => l.id === roadmap.next_lesson_id),
    );
    if (nextUnit) {
      setSelectedUnitNum(nextUnit.unit_number);
    }
  }, [roadmap]);

  const filteredUnits = useMemo<UnitSummary[]>(() => {
    if (!roadmap) return [];
    if (selectedStage === "all") return roadmap.units;
    return roadmap.units.filter((u) => u.stage === selectedStage);
  }, [roadmap, selectedStage]);

  const activeUnit = useMemo<UnitSummary | undefined>(() => {
    if (filteredUnits.length === 0) return undefined;
    return (
      filteredUnits.find((u) => u.unit_number === selectedUnitNum) ?? filteredUnits[0]
    );
  }, [filteredUnits, selectedUnitNum]);

  if (roadmapLoading && !roadmap) {
    return (
      <div className="flex min-h-[320px] flex-col items-center justify-center gap-3 rounded-2xl border border-border bg-card p-8 text-center">
        <Compass className="h-7 w-7 animate-spin text-primary" aria-hidden="true" />
        <p className="text-sm font-medium text-muted-foreground">{t.jungle.loading}</p>
      </div>
    );
  }

  if (roadmapError && !roadmap) {
    return (
      <div className="flex min-h-[280px] flex-col items-center justify-center gap-3 rounded-2xl border border-destructive/40 bg-card p-8 text-center">
        <p className="text-sm font-semibold text-foreground">{t.jungle.loadError}</p>
        <p className="max-w-md text-xs text-muted-foreground">{roadmapError}</p>
        <Button type="button" size="sm" onClick={() => void loadJungleRoadmap()}>
          <RefreshCw className="h-3.5 w-3.5" aria-hidden="true" />
          {t.jungle.retryLoad}
        </Button>
      </div>
    );
  }

  if (!roadmap) {
    return (
      <div className="flex min-h-[280px] flex-col items-center justify-center gap-3 rounded-2xl border border-border bg-card p-8 text-center">
        <p className="text-sm text-muted-foreground">{t.jungle.loadError}</p>
        <Button type="button" size="sm" onClick={() => void loadJungleRoadmap()}>
          <RefreshCw className="h-3.5 w-3.5" aria-hidden="true" />
          {t.jungle.retryLoad}
        </Button>
      </div>
    );
  }

  const pctCleared =
    roadmap.total_lessons > 0
      ? Math.round((roadmap.completed_lessons / roadmap.total_lessons) * 100)
      : 0;

  return (
    <div className="flex w-full flex-col gap-4">
      {/* Top Jungle Journey Overview & Continue CTA */}
      <section className="flex flex-col gap-4 rounded-2xl border border-border bg-card p-5 shadow-xs sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-1.5">
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="text-lg font-bold tracking-tight text-foreground">
              {t.jungle.title}
            </h2>
            <span className="rounded-full bg-muted px-2.5 py-0.5 text-xs font-medium text-muted-foreground">
              {t.layouts[roadmap.layout]}
            </span>
          </div>
          <p className="text-xs text-muted-foreground">{t.jungle.subtitle}</p>
          <div className="flex flex-wrap items-center gap-4 pt-1 text-xs">
            <span className="inline-flex items-center gap-1.5 font-mono font-semibold text-foreground">
              <Star className="h-3.5 w-3.5 fill-amber-400 text-amber-500" aria-hidden="true" />
              {roadmap.total_earned_stars.toLocaleString()} /{" "}
              {roadmap.total_max_stars.toLocaleString()}
            </span>
            <span className="font-mono text-muted-foreground">
              {roadmap.completed_lessons} / {roadmap.total_lessons} ({pctCleared}%)
            </span>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <Button
            type="button"
            className="min-h-11 px-4"
            onClick={() => void startJungleLesson(roadmap.next_lesson_id)}
          >
            <Play className="h-4 w-4" aria-hidden="true" />
            {t.jungle.continueNext} ({roadmap.next_lesson_id.toUpperCase()})
          </Button>
        </div>
      </section>

      {/* Curriculum Stage Filter Tabs */}
      <div
        role="tablist"
        aria-label={t.jungle.title}
        className="flex flex-wrap items-center gap-1.5 rounded-xl border border-border bg-muted/50 p-1.5"
      >
        {STAGES.map((stage) => {
          const active = selectedStage === stage;
          const label = stage === "all" ? t.jungle.allStages : t.jungle.stages[stage];
          return (
            <button
              key={stage}
              type="button"
              role="tab"
              aria-selected={active}
              onClick={() => {
                setSelectedStage(stage);
                const firstInStage =
                  stage === "all"
                    ? roadmap.units[0]
                    : roadmap.units.find((u) => u.stage === stage);
                if (firstInStage) setSelectedUnitNum(firstInStage.unit_number);
              }}
              className={cn(
                "min-h-9 rounded-lg px-3 py-1.5 text-xs font-semibold transition-colors",
                active
                  ? "bg-card text-primary shadow-2xs ring-1 ring-border"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              {label}
            </button>
          );
        })}
      </div>

      {/* Main Split: Left 28-Unit Selector Rail + Right Winding Lesson Path */}
      {filteredUnits.length === 0 ? (
        <div className="rounded-2xl border border-border bg-card p-8 text-center text-sm text-muted-foreground">
          {t.jungle.emptyUnits}
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-12">
          {/* Unit List */}
          <aside className="flex max-h-[540px] flex-col gap-2 overflow-y-auto rounded-2xl border border-border bg-card p-3 lg:col-span-4">
            {filteredUnits.map((unit) => {
              const selected = activeUnit?.unit_number === unit.unit_number;
              const unitTitle = uiLang === "hi" ? unit.title_hi : unit.title_en;
              return (
                <button
                  key={unit.unit_number}
                  type="button"
                  onClick={() => setSelectedUnitNum(unit.unit_number)}
                  className={cn(
                    "flex w-full items-center justify-between gap-2 rounded-xl border px-3 py-2.5 text-left transition-colors min-h-12",
                    selected
                      ? "border-primary bg-accent/65 text-foreground"
                      : unit.unlocked
                        ? "border-border/70 bg-background/60 hover:bg-muted/60 text-foreground"
                        : "border-border/50 bg-muted/30 text-muted-foreground",
                  )}
                >
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-[11px] font-bold text-primary">
                        U{String(unit.unit_number).padStart(2, "0")}
                      </span>
                      {!unit.unlocked && (
                        <Lock className="h-3 w-3 shrink-0 text-muted-foreground" aria-hidden="true" />
                      )}
                      <span className="truncate text-xs font-semibold">{unitTitle}</span>
                    </div>
                    <div className="mt-1 flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
                      <span>
                        {unit.completed_lessons}/{unit.lessons.length}
                      </span>
                      <span>·</span>
                      <span className={cn(isHindi && "font-[family-name:var(--font-devanagari)]")}>
                        {unit.focus_keys.slice(0, 4).join(" ")}
                      </span>
                    </div>
                  </div>

                  <div className="flex shrink-0 items-center gap-1 font-mono text-xs font-semibold text-foreground">
                    <Star className="h-3.5 w-3.5 fill-amber-400 text-amber-500" aria-hidden="true" />
                    {unit.earned_stars}/{unit.max_stars}
                  </div>
                </button>
              );
            })}
          </aside>

          {/* Active Unit Winding Trail */}
          {activeUnit && (
            <section className="flex flex-col gap-4 rounded-2xl border border-border bg-card p-5 shadow-xs lg:col-span-8">
              <div className="flex flex-wrap items-start justify-between gap-3 border-b border-border pb-4">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="rounded-md bg-primary/15 px-2 py-0.5 font-mono text-xs font-bold text-primary">
                      {t.jungle.unitLabel} {activeUnit.unit_number}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {t.jungle.stages[activeUnit.stage]}
                    </span>
                  </div>
                  <h3 className="mt-1.5 text-base font-bold text-foreground">
                    {uiLang === "hi" ? activeUnit.title_hi : activeUnit.title_en}
                  </h3>
                </div>

                <div className="flex flex-wrap items-center gap-3 text-xs">
                  <div className="rounded-lg border border-border bg-background/70 px-2.5 py-1.5">
                    <span className="text-muted-foreground">{t.jungle.focusKeys}: </span>
                    <span
                      className={cn(
                        "font-bold text-foreground",
                        isHindi ? "font-[family-name:var(--font-devanagari)] text-sm" : "font-mono",
                      )}
                    >
                      {activeUnit.focus_keys.join(" ")}
                    </span>
                  </div>
                  <div className="rounded-lg border border-border bg-background/70 px-2.5 py-1.5 font-mono">
                    {Math.round(activeUnit.target_wpm ?? 0)} WPM ·{" "}
                    {Math.round(activeUnit.min_accuracy ?? 0)}%
                  </div>
                </div>
              </div>

              <SerpentineLessonGrid
                lessons={activeUnit.lessons}
                nextLessonId={roadmap.next_lesson_id}
                uiLang={uiLang}
                onSelectLesson={(lessonId) => void startJungleLesson(lessonId)}
              />
            </section>
          )}
        </div>
      )}
    </div>
  );
}

function SerpentineLessonGrid({
  lessons,
  nextLessonId,
  uiLang,
  onSelectLesson,
}: {
  lessons: LessonNodeSummary[];
  nextLessonId: string;
  uiLang: "en" | "hi";
  onSelectLesson: (lessonId: string) => void;
}) {
  const t = getDictionary(uiLang);

  // Arrange 25 lessons in 5 rows of 5, alternating left-to-right and right-to-left like a jungle trail.
  const rows = useMemo(() => {
    const chunked: { items: LessonNodeSummary[]; reversed: boolean }[] = [];
    for (let i = 0; i < lessons.length; i += 5) {
      const slice = lessons.slice(i, i + 5);
      const rowIdx = Math.floor(i / 5);
      const reversed = rowIdx % 2 === 1;
      chunked.push({ items: reversed ? [...slice].reverse() : slice, reversed });
    }
    return chunked;
  }, [lessons]);

  return (
    <div className="relative space-y-3" role="list" aria-label={t.jungle.lessonLabel}>
      {rows.map((row, rowIdx) => (
        <div
          key={rowIdx}
          className="grid grid-cols-2 gap-2.5 sm:grid-cols-5"
        >
          {row.items.map((lesson) => {
            const Icon = lessonKindIcon(lesson.kind);
            const isNext = lesson.id === nextLessonId;
            const isMiniGame = lesson.kind === "mini_game";
            const isBoss = lesson.kind === "review_challenge";
            const kindLabel = t.jungle.kinds[lesson.kind];
            const statusText = lesson.unlocked ? t.jungle.unlocked : t.jungle.locked;

            return (
              <button
                key={lesson.id}
                type="button"
                role="listitem"
                disabled={!lesson.unlocked}
                title={lesson.unlocked ? `${kindLabel} · #${lesson.global_number}` : t.jungle.lockedHint}
                aria-label={`${t.jungle.lessonLabel} ${lesson.global_number}: ${kindLabel}, ${lesson.stars} of 5 ${t.jungle.starsLabel}, ${statusText}`}
                onClick={() => onSelectLesson(lesson.id)}
                className={cn(
                  "group relative flex min-h-[78px] flex-col justify-between rounded-xl border p-2.5 text-left transition-all",
                  !lesson.unlocked &&
                    "border-border/50 bg-muted/35 text-muted-foreground opacity-70",
                  lesson.unlocked &&
                    !isNext &&
                    lesson.stars >= 3 &&
                    "border-emerald-600/40 bg-emerald-500/8 hover:border-emerald-600/70",
                  lesson.unlocked &&
                    !isNext &&
                    lesson.stars < 3 &&
                    "border-border bg-background/80 hover:border-primary/60",
                  isNext &&
                    "border-primary bg-accent/65 ring-2 ring-primary/30",
                  isMiniGame && lesson.unlocked && "border-amber-600/50",
                  isBoss && lesson.unlocked && "border-amber-700/60",
                )}
              >
                <div className="flex items-center justify-between gap-1">
                  <span className="font-mono text-[11px] font-bold text-muted-foreground">
                    #{lesson.global_number}
                  </span>
                  {lesson.unlocked ? (
                    <Icon
                      className={cn(
                        "h-3.5 w-3.5",
                        isNext
                          ? "text-primary"
                          : isMiniGame || isBoss
                            ? "text-amber-600 dark:text-amber-400"
                            : "text-muted-foreground",
                      )}
                      aria-hidden="true"
                    />
                  ) : (
                    <Lock className="h-3.5 w-3.5 text-muted-foreground" aria-hidden="true" />
                  )}
                </div>

                <div className="truncate text-[11px] font-semibold text-foreground">
                  {kindLabel}
                </div>

                <div className="flex items-center justify-between gap-1 pt-0.5">
                  <div className="flex items-center gap-0.5" aria-hidden="true">
                    {[1, 2, 3, 4, 5].map((starIdx) => (
                      <Star
                        key={starIdx}
                        className={cn(
                          "h-2.5 w-2.5",
                          starIdx <= lesson.stars
                            ? "fill-amber-400 text-amber-500"
                            : "text-muted-foreground/30",
                        )}
                      />
                    ))}
                  </div>
                  {lesson.best_wpm ? (
                    <span className="font-mono text-[10px] text-muted-foreground">
                      {Math.round(lesson.best_wpm)}w
                    </span>
                  ) : null}
                </div>
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
}
