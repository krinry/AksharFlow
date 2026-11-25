import { useEffect } from "react";
import { TopBar } from "@/components/layout/TopBar";
import { MetricsStrip } from "@/components/typing/MetricsStrip";
import { UnlockProgressBar } from "@/components/typing/UnlockProgressBar";
import { TypingStage } from "@/components/typing/TypingStage";
import { VirtualKeyboard } from "@/components/keyboard/VirtualKeyboard";
import { UnlockToast } from "@/components/typing/UnlockToast";
import { SettingsBackupModal } from "@/components/settings/SettingsBackupModal";
import { useTypingStore } from "@/stores/useTypingStore";

export function App() {
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

      <main className="mx-auto flex w-full max-w-5xl flex-1 flex-col justify-center gap-4 px-4 py-5 sm:px-6">
        <MetricsStrip />
        {!zenMode && <UnlockProgressBar />}
        <TypingStage />
        {!zenMode && <VirtualKeyboard />}
      </main>

      <UnlockToast />
      <SettingsBackupModal />
    </div>
  );
}

export default App;
