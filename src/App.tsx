import { useEffect } from "react";
import { TopBar } from "@/components/layout/TopBar";
import { MetricsStrip } from "@/components/typing/MetricsStrip";
import { UnlockProgressBar } from "@/components/typing/UnlockProgressBar";
import { TypingStage } from "@/components/typing/TypingStage";
import { VirtualKeyboard } from "@/components/keyboard/VirtualKeyboard";
import { FingerGuideSvg } from "@/components/keyboard/FingerGuideSvg";
import { UnlockToast } from "@/components/typing/UnlockToast";
import { SettingsBackupModal } from "@/components/settings/SettingsBackupModal";
import { ExamPanel } from "@/components/exam/ExamPanel";
import { ExamReportModal } from "@/components/exam/ExamReportModal";
import { AnalyticsDashboard } from "@/components/analytics/AnalyticsDashboard";
import { JungleMap } from "@/components/jungle/JungleMap";
import { BalloonMiniGame } from "@/components/jungle/BalloonMiniGame";
import { LessonCompleteModal } from "@/components/jungle/LessonCompleteModal";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";
import { cn } from "@/lib/utils";

export function App() {
  const activeTab = useTypingStore((s) => s.activeTab);
  const practiceMode = useTypingStore((s) => s.practiceMode);
  const activeLessonMeta = useTypingStore((s) => s.activeLessonMeta);
  const layout = useTypingStore((s) => s.layout);
  const uiLang = useTypingStore((s) => s.uiLang);
  const theme = useTypingStore((s) => s.theme);
  const zenMode = useTypingStore((s) => s.zenMode);
  const initApp = useTypingStore((s) => s.initApp);

  const t = getDictionary(uiLang);
  const isHindi = layout !== "en_qwerty";

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("dark", theme === "dark");
  }, [theme]);

  useEffect(() => {
    void initApp();
  }, [initApp]);

  const showJungleMap = activeTab === "jungle" && !activeLessonMeta;
  const showMiniGame = activeTab === "jungle" && practiceMode === "mini_game";

  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground transition-colors">
      <TopBar />

      <main className="mx-auto flex w-full max-w-6xl flex-1 flex-col justify-center gap-4 px-4 py-5 sm:px-6">
        {activeTab === "analytics" ? (
          <AnalyticsDashboard />
        ) : showJungleMap ? (
          <JungleMap />
        ) : showMiniGame ? (
          <BalloonMiniGame />
        ) : (
          <>
            {!zenMode && activeTab === "exam" && <ExamPanel />}
            {!zenMode && activeTab === "jungle" && activeLessonMeta && (
              <section className="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-border bg-card px-4 py-3 shadow-2xs">
                <div className="flex flex-wrap items-center gap-2.5">
                  <span className="rounded-md bg-primary/15 px-2 py-0.5 font-mono text-xs font-bold text-primary">
                    {activeLessonMeta.id.toUpperCase()}
                  </span>
                  <h2 className="text-sm font-bold text-foreground">
                    {uiLang === "hi" ? activeLessonMeta.title_hi : activeLessonMeta.title_en}
                  </h2>
                  <span className="text-xs text-muted-foreground">
                    {t.jungle.kinds[activeLessonMeta.kind]}
                  </span>
                </div>
                <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
                  <span>
                    {t.jungle.focusKeys}:{" "}
                    <strong
                      className={cn(
                        "text-foreground",
                        isHindi
                          ? "font-[family-name:var(--font-devanagari)] text-sm"
                          : "font-mono",
                      )}
                    >
                      {activeLessonMeta.focus_keys.join(" ")}
                    </strong>
                  </span>
                  <span className="font-mono">
                    {t.jungle.targetGoal}: {Math.round(activeLessonMeta.target_wpm ?? 0)} WPM ·{" "}
                    {Math.round(activeLessonMeta.min_accuracy ?? 0)}%
                  </span>
                </div>
              </section>
            )}
            {!zenMode && <MetricsStrip />}
            {!zenMode && activeTab === "adaptive" && <UnlockProgressBar />}
            <TypingStage />
            {!zenMode && (
              <div className="flex flex-col gap-3 xl:flex-row xl:items-stretch">
                <div className="flex-1">
                  <VirtualKeyboard />
                </div>
                <FingerGuideSvg />
              </div>
            )}
          </>
        )}
      </main>

      <UnlockToast />
      <SettingsBackupModal />
      <ExamReportModal />
      <LessonCompleteModal />
    </div>
  );
}

export default App;
