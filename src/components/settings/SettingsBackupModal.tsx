import { useEffect, useState } from "react";
import { Download, RotateCcw, Upload, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";

export function SettingsBackupModal() {
  const open = useTypingStore((s) => s.settingsModalOpen);
  const setOpen = useTypingStore((s) => s.setSettingsModalOpen);
  const settings = useTypingStore((s) => s.settings);
  const uiLang = useTypingStore((s) => s.uiLang);
  const updateAdaptiveSettings = useTypingStore((s) => s.updateAdaptiveSettings);
  const exportProgressBackup = useTypingStore((s) => s.exportProgressBackup);
  const importProgressBackup = useTypingStore((s) => s.importProgressBackup);
  const resetCurrentLayout = useTypingStore((s) => s.resetCurrentLayout);

  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const [confirmReset, setConfirmReset] = useState(false);

  const t = getDictionary(uiLang);

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        setConfirmReset(false);
        setStatusMsg(null);
        setOpen(false);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, setOpen]);

  if (!open) return null;

  const handleExport = async () => {
    try {
      const path = await exportProgressBackup();
      if (path) {
        setStatusMsg(`${t.settings.exportSuccess} (${path})`);
      }
    } catch (err) {
      setStatusMsg(String(err));
    }
  };

  const handleImport = async (replace: boolean) => {
    try {
      const ok = await importProgressBackup(replace);
      if (ok) {
        setStatusMsg(t.settings.importSuccess);
      }
    } catch (err) {
      setStatusMsg(String(err));
    }
  };

  const handleReset = async () => {
    if (!confirmReset) {
      setConfirmReset(true);
      return;
    }
    await resetCurrentLayout();
    setConfirmReset(false);
    setStatusMsg(t.settings.resetSuccess);
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-modal-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-xs"
    >
      <div className="w-full max-w-lg rounded-2xl border border-border bg-card p-6 shadow-xl">
        <div className="mb-4 flex items-center justify-between">
          <h2 id="settings-modal-title" className="text-base font-bold">
            {t.settings.title}
          </h2>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            aria-label={t.settings.close}
            onClick={() => {
              setConfirmReset(false);
              setStatusMsg(null);
              setOpen(false);
            }}
          >
            <X className="h-4 w-4" aria-hidden="true" />
          </Button>
        </div>

        <div className="space-y-4 text-sm">
          <SliderField
            label={t.settings.targetWpm}
            value={Math.round(settings.target_wpm ?? 25)}
            min={15}
            max={80}
            step={5}
            unit="WPM"
            onChange={(val) => void updateAdaptiveSettings({ target_wpm: val })}
          />
          <SliderField
            label={t.settings.targetAccuracy}
            value={Math.round(settings.target_accuracy ?? 95)}
            min={80}
            max={100}
            step={1}
            unit="%"
            onChange={(val) => void updateAdaptiveSettings({ target_accuracy: val })}
          />
          <SliderField
            label={t.settings.unlockThreshold}
            value={Math.round(settings.unlock_threshold ?? 85)}
            min={70}
            max={98}
            step={1}
            unit="%"
            onChange={(val) => void updateAdaptiveSettings({ unlock_threshold: val })}
          />
          <SliderField
            label={t.settings.volume}
            value={Math.round((settings.sound_volume ?? 0.55) * 100)}
            min={0}
            max={100}
            step={5}
            unit="%"
            onChange={(val) => void updateAdaptiveSettings({ sound_volume: val / 100 })}
          />

          <div className="border-t border-border pt-4">
            <h3 className="mb-2.5 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {t.settings.backupTitle}
            </h3>
            <div className="flex flex-wrap gap-2">
              <Button variant="default" size="sm" onClick={() => void handleExport()}>
                <Download className="h-3.5 w-3.5" />
                {t.settings.exportJson}
              </Button>
              <Button variant="outline" size="sm" onClick={() => void handleImport(false)}>
                <Upload className="h-3.5 w-3.5" />
                {t.settings.importMerge}
              </Button>
              <Button variant="outline" size="sm" onClick={() => void handleImport(true)}>
                <Upload className="h-3.5 w-3.5" />
                {t.settings.importReplace}
              </Button>
            </div>
          </div>

          <div className="border-t border-border pt-4 flex items-center justify-between gap-3">
            <Button variant="secondary" size="sm" onClick={() => void handleReset()}>
              <RotateCcw className="h-3.5 w-3.5" />
              {confirmReset ? t.settings.confirmReset : t.settings.resetLayout}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                setConfirmReset(false);
                setStatusMsg(null);
                setOpen(false);
              }}
            >
              {t.settings.close}
            </Button>
          </div>

          {statusMsg && (
            <div className="rounded-lg border border-border bg-muted px-3 py-2 text-xs text-foreground">
              {statusMsg}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function SliderField({
  label,
  value,
  min,
  max,
  step,
  unit,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  unit: string;
  onChange: (v: number) => void;
}) {
  return (
    <label className="block space-y-1">
      <div className="flex items-center justify-between text-xs font-medium">
        <span>{label}</span>
        <span className="font-mono font-bold text-primary">
          {value} {unit}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full accent-amber-500 cursor-pointer"
      />
    </label>
  );
}
