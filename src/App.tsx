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
import { useTypingStore } from "@/stores/useTypingStore";

export function App() {
  const activeTab = useTypingStore((s) => s.activeTab);
  const theme = useTypingStore((s) => s.theme);
  const zenMode = useTypingStore((s) => s.zenMode);
  const initApp = useTypingStore((s) => s.initApp);

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("dark", theme === "dark");
  }, [theme]);

  useEffect(() => {
    void initApp();
  }, [initApp]);

  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground transition-colors">
      <TopBar />

      <main className="mx-auto flex w-full max-w-6xl flex-1 flex-col justify-center gap-4 px-4 py-5 sm:px-6">
        {activeTab === "analytics" ? (
          <AnalyticsDashboard />
        ) : (
          <>
            {!zenMode && activeTab === "exam" && <ExamPanel />}
            {!zenMode && <MetricsStrip />}
            {!zenMode && activeTab === "practice" && <UnlockProgressBar />}
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
    </div>
  );
}

export default App;
