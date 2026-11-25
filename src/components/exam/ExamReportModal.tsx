import { useEffect } from "react";
import { Award, RotateCcw, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";

export function ExamReportModal() {
  const examSummary = useTypingStore((s) => s.examSummary);
  const uiLang = useTypingStore((s) => s.uiLang);
  const dismissExamSummary = useTypingStore((s) => s.dismissExamSummary);
  const startExamWithSelected = useTypingStore((s) => s.startExamWithSelected);
  const setActiveTab = useTypingStore((s) => s.setActiveTab);

  const t = getDictionary(uiLang);

  useEffect(() => {
    if (!examSummary) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        dismissExamSummary();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [examSummary, dismissExamSummary]);

  if (!examSummary) return null;

  const m = examSummary.metrics;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="exam-report-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/55 p-4 backdrop-blur-xs"
    >
      <div className="w-full max-w-lg rounded-2xl border border-border bg-card p-6 shadow-xl">
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-primary/15 text-primary">
              <Award className="h-5 w-5" aria-hidden="true" />
            </div>
            <div>
              <h2 id="exam-report-title" className="text-base font-bold">
                {t.exam.reportTitle}
              </h2>
              <p className="text-xs text-muted-foreground">{t.layouts[examSummary.layout]}</p>
            </div>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            aria-label={t.settings.close}
            onClick={dismissExamSummary}
          >
            <X className="h-4 w-4" aria-hidden="true" />
          </Button>
        </div>

        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
          <StatTile label={t.metrics.netWpm} value={String(Math.round(m.net_wpm ?? 0))} accent />
          <StatTile label={t.metrics.grossWpm} value={String(Math.round(m.gross_wpm ?? 0))} />
          <StatTile label={t.metrics.kph} value={String(Math.round(m.kph ?? 0))} />
          <StatTile label={t.metrics.accuracy} value={`${(m.accuracy ?? 100).toFixed(1)}%`} />
          <StatTile label={t.metrics.consistency} value={`${Math.round(m.consistency ?? 100)}%`} />
          <StatTile label={t.metrics.errors} value={String(m.errors)} />
          <StatTile label={t.exam.keystrokes} value={String(m.keystrokes)} />
          <StatTile label={t.exam.linesDone} value={String(examSummary.lines_completed)} />
          <StatTile
            label={t.metrics.time}
            value={`${Math.round(m.elapsed_ms / 1000)}s`}
          />
        </div>

        <div className="mt-5 flex flex-wrap justify-end gap-2 border-t border-border pt-4">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => {
              dismissExamSummary();
              void setActiveTab("adaptive");
            }}
          >
            {t.exam.backToPractice}
          </Button>
          <Button
            variant="default"
            size="sm"
            onClick={() => {
              dismissExamSummary();
              void startExamWithSelected();
            }}
          >
            <RotateCcw className="h-3.5 w-3.5" />
            {t.exam.retryExam}
          </Button>
        </div>
      </div>
    </div>
  );
}

function StatTile({
  label,
  value,
  accent = false,
}: {
  label: string;
  value: string;
  accent?: boolean;
}) {
  return (
    <div className="rounded-xl border border-border bg-background/70 px-3 py-2.5">
      <div className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
        {label}
      </div>
      <div
        className={
          accent
            ? "mt-0.5 font-mono text-2xl font-bold text-primary"
            : "mt-0.5 font-mono text-lg font-bold text-foreground"
        }
      >
        {value}
      </div>
    </div>
  );
}
