import { useEffect, useState } from "react";
import { Clock, FilePlus2, Flag, Play, ShieldAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";
import { cn } from "@/lib/utils";

const DURATIONS = [1, 2, 5, 10, 15];

function formatClock(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

export function ExamPanel() {
  const layout = useTypingStore((s) => s.layout);
  const uiLang = useTypingStore((s) => s.uiLang);
  const examPassages = useTypingStore((s) => s.examPassages);
  const selectedPassageId = useTypingStore((s) => s.selectedPassageId);
  const examDurationMins = useTypingStore((s) => s.examDurationMins);
  const examAllowBackspace = useTypingStore((s) => s.examAllowBackspace);
  const examRemainingSecs = useTypingStore((s) => s.examRemainingSecs);
  const examTimerRunning = useTypingStore((s) => s.examTimerRunning);
  const metrics = useTypingStore((s) => s.metrics);
  const setExamConfig = useTypingStore((s) => s.setExamConfig);
  const startExamWithSelected = useTypingStore((s) => s.startExamWithSelected);
  const saveCustomPassage = useTypingStore((s) => s.saveCustomPassage);
  const tickExamTimer = useTypingStore((s) => s.tickExamTimer);
  const finishExam = useTypingStore((s) => s.finishExam);

  const [showCustomForm, setShowCustomForm] = useState(false);
  const [customTitle, setCustomTitle] = useState("");
  const [customContent, setCustomContent] = useState("");

  const t = getDictionary(uiLang);
  const layoutLang = layout === "en_qwerty" ? "en" : "hi";

  useEffect(() => {
    if (!examTimerRunning) return;
    const id = window.setInterval(() => {
      tickExamTimer();
    }, 1000);
    return () => window.clearInterval(id);
  }, [examTimerRunning, tickExamTimer]);

  const sortedPassages = [...examPassages].sort((a, b) => {
    const aMatch = a.language === layoutLang ? 0 : 1;
    const bMatch = b.language === layoutLang ? 0 : 1;
    return aMatch - bMatch;
  });

  const handleSaveCustom = async () => {
    if (!customTitle.trim() || !customContent.trim()) return;
    await saveCustomPassage(customTitle.trim(), layoutLang, customContent.trim());
    setCustomTitle("");
    setCustomContent("");
    setShowCustomForm(false);
  };

  return (
    <section className="w-full rounded-2xl border border-border bg-card/85 p-4 shadow-sm">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex items-center gap-1.5 text-sm font-bold">
            <Clock className="h-4 w-4 text-primary" />
            <span>{t.exam.title}</span>
          </div>

          {/* Duration selector */}
          <div className="flex items-center rounded-xl border border-border bg-muted/60 p-1">
            {DURATIONS.map((mins) => (
              <button
                key={mins}
                type="button"
                onClick={() => {
                  setExamConfig({ examDurationMins: mins });
                  void startExamWithSelected();
                }}
                className={cn(
                  "rounded-lg px-2.5 py-1 text-xs font-semibold transition-all cursor-pointer",
                  examDurationMins === mins
                    ? "bg-card text-primary shadow-2xs ring-1 ring-border"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                {mins} {t.exam.minutes}
              </button>
            ))}
          </div>

          {/* Strict Backspace toggle */}
          <button
            type="button"
            onClick={() => {
              const next = !examAllowBackspace;
              setExamConfig({ examAllowBackspace: next });
              void startExamWithSelected();
            }}
            className={cn(
              "inline-flex items-center gap-1.5 rounded-xl border px-3 py-1.5 text-xs font-medium transition-colors cursor-pointer",
              examAllowBackspace
                ? "border-border bg-background text-foreground hover:bg-muted"
                : "border-amber-500/50 bg-amber-500/15 text-amber-700 dark:text-amber-300",
            )}
          >
            <ShieldAlert className="h-3.5 w-3.5" />
            {examAllowBackspace ? t.exam.allowBackspace : t.exam.strictMode}
          </button>
        </div>

        {/* Countdown Clock + Submit */}
        <div className="flex items-center gap-2.5">
          {!examTimerRunning && metrics.keystrokes === 0 && (
            <Badge variant="default">{t.exam.waitingStart}</Badge>
          )}
          <div
            className={cn(
              "rounded-xl border px-3.5 py-1.5 font-mono text-lg font-bold tabular-nums",
              examRemainingSecs <= 15
                ? "border-destructive/50 bg-destructive/15 text-destructive"
                : examRemainingSecs <= 60
                  ? "border-amber-500/50 bg-amber-500/15 text-amber-600 dark:text-amber-300"
                  : "border-border bg-background text-foreground",
            )}
          >
            {formatClock(examRemainingSecs)}
          </div>
          <Button variant="secondary" size="sm" onClick={() => void startExamWithSelected()}>
            <Play className="h-3.5 w-3.5" />
            {t.exam.startExam}
          </Button>
          <Button
            variant="default"
            size="sm"
            disabled={metrics.keystrokes === 0}
            onClick={() => void finishExam()}
          >
            <Flag className="h-3.5 w-3.5" />
            {t.exam.submitExam}
          </Button>
        </div>
      </div>

      {/* Passage selector + Custom Passage button */}
      <div className="mt-3 flex flex-wrap items-center gap-2 border-t border-border/60 pt-3">
        <span className="text-xs font-medium text-muted-foreground">{t.exam.passage}:</span>
        <select
          value={selectedPassageId}
          onChange={(e) => {
            const id = e.target.value;
            const found = examPassages.find((p) => p.id === id);
            setExamConfig({ selectedPassageId: id });
            if (found) {
              void startExamWithSelected(found.content);
            }
          }}
          className="min-w-[260px] flex-1 rounded-lg border border-border bg-background px-3 py-1.5 text-xs font-medium text-foreground focus:outline-none cursor-pointer"
        >
          {sortedPassages.map((p) => (
            <option key={p.id} value={p.id}>
              [{p.language.toUpperCase()} · {p.category}] {p.title}
            </option>
          ))}
        </select>

        <Button
          variant="outline"
          size="sm"
          onClick={() => setShowCustomForm((v) => !v)}
        >
          <FilePlus2 className="h-3.5 w-3.5" />
          {t.exam.addCustom}
        </Button>
      </div>

      {showCustomForm && (
        <div className="mt-3 space-y-2 rounded-xl border border-border bg-background/80 p-3">
          <input
            type="text"
            value={customTitle}
            onChange={(e) => setCustomTitle(e.target.value)}
            placeholder={t.exam.customTitle}
            className="w-full rounded-lg border border-border bg-card px-3 py-1.5 text-xs focus:outline-none focus:ring-2 focus:ring-primary/40"
          />
          <textarea
            rows={3}
            value={customContent}
            onChange={(e) => setCustomContent(e.target.value)}
            placeholder={t.exam.customText}
            className="w-full rounded-lg border border-border bg-card px-3 py-2 text-xs focus:outline-none focus:ring-2 focus:ring-primary/40"
          />
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setShowCustomForm(false)}>
              {t.exam.cancel}
            </Button>
            <Button variant="default" size="sm" onClick={() => void handleSaveCustom()}>
              {t.exam.savePassage}
            </Button>
          </div>
        </div>
      )}
    </section>
  );
}
