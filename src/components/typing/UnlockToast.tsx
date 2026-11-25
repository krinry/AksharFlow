import { useEffect } from "react";
import { Sparkles, X } from "lucide-react";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";

export function UnlockToast() {
  const unlockedToast = useTypingStore((s) => s.unlockedToast);
  const uiLang = useTypingStore((s) => s.uiLang);
  const dismissUnlockToast = useTypingStore((s) => s.dismissUnlockToast);

  const t = getDictionary(uiLang);

  useEffect(() => {
    if (!unlockedToast) return;
    const timer = window.setTimeout(() => {
      dismissUnlockToast();
    }, 4500);
    return () => window.clearTimeout(timer);
  }, [unlockedToast, dismissUnlockToast]);

  if (!unlockedToast) return null;

  return (
    <div className="fixed bottom-5 right-5 z-50 flex items-center gap-3 rounded-2xl border border-emerald-500/50 bg-card px-4 py-3 shadow-lg ring-2 ring-emerald-500/20">
      <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-emerald-500/15 text-emerald-500">
        <Sparkles className="h-5 w-5" />
      </div>
      <div>
        <div className="text-xs font-semibold uppercase tracking-wider text-emerald-600 dark:text-emerald-400">
          {t.adaptive.keyUnlockedTitle}
        </div>
        <div className="flex items-center gap-2 text-sm font-bold">
          <span className="font-[family-name:var(--font-devanagari)] text-base">
            ‘{unlockedToast.key_id}’
          </span>
          <span className="text-xs font-normal text-muted-foreground">
            ({unlockedToast.physical_code} · {t.fingers[unlockedToast.finger]})
          </span>
        </div>
      </div>
      <button
        type="button"
        onClick={dismissUnlockToast}
        className="ml-2 rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}
