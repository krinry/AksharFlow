import { useEffect } from "react";
import { TopBar } from "@/components/layout/TopBar";
import { MetricsStrip } from "@/components/typing/MetricsStrip";
import { TypingStage } from "@/components/typing/TypingStage";
import { VirtualKeyboard } from "@/components/keyboard/VirtualKeyboard";
import { useTypingStore } from "@/stores/useTypingStore";

export function App() {
  const theme = useTypingStore((s) => s.theme);
  const zenMode = useTypingStore((s) => s.zenMode);
  const initSession = useTypingStore((s) => s.initSession);

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("dark", theme === "dark");
  }, [theme]);

  useEffect(() => {
    void initSession();
  }, [initSession]);

  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground transition-colors">
      <TopBar />

      <main className="mx-auto flex w-full max-w-5xl flex-1 flex-col justify-center gap-5 px-4 py-6 sm:px-6">
        <MetricsStrip />
        <TypingStage />
        {!zenMode && <VirtualKeyboard />}
      </main>
    </div>
  );
}

export default App;
