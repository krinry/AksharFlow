import { Globe, Keyboard, Maximize2, Minimize2, Moon, Sun, Volume2, VolumeX } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { LayoutKind } from "@/lib/bindings";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";
import type { SoundProfile } from "@/audio/soundEngine";
import { cn } from "@/lib/utils";

const LAYOUTS: LayoutKind[] = ["en_qwerty", "hi_inscript", "hi_remington_gail"];
const SOUND_PROFILES: SoundProfile[] = ["off", "blue", "brown", "red", "topre"];

export function TopBar() {
  const layout = useTypingStore((s) => s.layout);
  const uiLang = useTypingStore((s) => s.uiLang);
  const theme = useTypingStore((s) => s.theme);
  const soundProfile = useTypingStore((s) => s.soundProfile);
  const zenMode = useTypingStore((s) => s.zenMode);
  const setLayout = useTypingStore((s) => s.setLayout);
  const setUiLang = useTypingStore((s) => s.setUiLang);
  const toggleTheme = useTypingStore((s) => s.toggleTheme);
  const setSoundProfile = useTypingStore((s) => s.setSoundProfile);
  const toggleZenMode = useTypingStore((s) => s.toggleZenMode);

  const t = getDictionary(uiLang);

  if (zenMode) {
    return (
      <header className="flex items-center justify-end px-6 pt-4">
        <Button variant="ghost" size="sm" onClick={toggleZenMode}>
          <Minimize2 className="h-3.5 w-3.5" />
          {t.typing.exitZen}
        </Button>
      </header>
    );
  }

  return (
    <header className="flex flex-wrap items-center justify-between gap-3 border-b border-border/70 bg-card/65 px-5 py-3 backdrop-blur-sm">
      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-primary/15 text-primary ring-1 ring-primary/30">
          <Keyboard className="h-5 w-5" />
        </div>
        <div>
          <div className="flex items-baseline gap-2">
            <h1 className="text-base font-bold tracking-tight">{t.appName}</h1>
            <span className="text-xs font-medium text-muted-foreground">अक्षर-फ्लो</span>
          </div>
          <p className="text-[11px] text-muted-foreground">{t.tagline}</p>
        </div>
      </div>

      <div className="flex items-center rounded-xl border border-border bg-muted/60 p-1">
        {LAYOUTS.map((item) => {
          const active = layout === item;
          return (
            <button
              key={item}
              type="button"
              onClick={() => void setLayout(item)}
              className={cn(
                "rounded-lg px-3 py-1.5 text-xs font-medium transition-all cursor-pointer",
                active
                  ? "bg-card text-foreground shadow-xs ring-1 ring-border/70"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              {t.layouts[item]}
            </button>
          );
        })}
      </div>

      <div className="flex items-center gap-2">
        <div className="flex items-center gap-1 rounded-lg border border-border bg-card px-2 py-1">
          {soundProfile === "off" ? (
            <VolumeX className="h-3.5 w-3.5 text-muted-foreground" />
          ) : (
            <Volume2 className="h-3.5 w-3.5 text-primary" />
          )}
          <select
            aria-label={t.sound.label}
            value={soundProfile}
            onChange={(e) => setSoundProfile(e.target.value as SoundProfile)}
            className="bg-transparent text-xs font-medium text-foreground focus:outline-none cursor-pointer"
          >
            {SOUND_PROFILES.map((p) => (
              <option key={p} value={p} className="bg-card text-foreground">
                {t.sound[p]}
              </option>
            ))}
          </select>
        </div>

        <Button
          variant="outline"
          size="sm"
          onClick={() => setUiLang(uiLang === "en" ? "hi" : "en")}
          title="Switch UI Language"
        >
          <Globe className="h-3.5 w-3.5" />
          {uiLang === "en" ? "हिं" : "EN"}
        </Button>

        <Button
          variant="outline"
          size="icon"
          onClick={toggleTheme}
          title={theme === "dark" ? t.theme.light : t.theme.dark}
        >
          {theme === "dark" ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
        </Button>

        <Button variant="outline" size="sm" onClick={toggleZenMode}>
          <Maximize2 className="h-3.5 w-3.5" />
          {t.typing.zenMode}
        </Button>
      </div>
    </header>
  );
}
