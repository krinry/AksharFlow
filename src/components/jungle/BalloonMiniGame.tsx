import { useCallback, useEffect, useRef, useState } from "react";
import { Gamepad2, Heart, Map, Pause, Play, RotateCcw, Trophy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";
import { cn } from "@/lib/utils";

interface FallingBalloon {
  id: number;
  word: string;
  xRatio: number;
  yPx: number;
  speedPxPerSec: number;
}

interface PopParticle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
}

const MAX_LIVES = 5;

export function BalloonMiniGame() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const stageRef = useRef<HTMLDivElement>(null);

  const layout = useTypingStore((s) => s.layout);
  const uiLang = useTypingStore((s) => s.uiLang);
  const theme = useTypingStore((s) => s.theme);
  const activeLessonMeta = useTypingStore((s) => s.activeLessonMeta);
  const miniGameWords = useTypingStore((s) => s.miniGameWords);
  const lessonCompletion = useTypingStore((s) => s.lessonCompletion);
  const metrics = useTypingStore((s) => s.metrics);
  const pendingImePreview = useTypingStore((s) => s.pendingImePreview);
  const resolveGameKeystroke = useTypingStore((s) => s.resolveGameKeystroke);
  const completeJungleLesson = useTypingStore((s) => s.completeJungleLesson);
  const restartActiveLesson = useTypingStore((s) => s.restartActiveLesson);
  const exitActiveLessonToMap = useTypingStore((s) => s.exitActiveLessonToMap);
  const setModifiers = useTypingStore((s) => s.setModifiers);

  const t = getDictionary(uiLang);
  const isHindi = layout !== "en_qwerty";

  const words = miniGameWords.length > 0 ? miniGameWords : ["akshar", "flow", "type"];
  const [paused, setPaused] = useState(false);
  const [score, setScore] = useState(0);
  const [lives, setLives] = useState(MAX_LIVES);
  const [poppedCount, setPoppedCount] = useState(0);
  const [missedCount, setMissedCount] = useState(0);
  const [lockedWord, setLockedWord] = useState<string | null>(null);
  const [typedPreview, setTypedPreview] = useState("");
  const [inputError, setInputError] = useState(false);
  const [reducedMotion, setReducedMotion] = useState(false);

  const balloonsRef = useRef<FallingBalloon[]>([]);
  const particlesRef = useRef<PopParticle[]>([]);
  const nextWordIdxRef = useRef(0);
  const nextIdRef = useRef(1);
  const spawnAccumMsRef = useRef(1500);
  const livesRef = useRef(MAX_LIVES);
  const poppedRef = useRef(0);
  const missedRef = useRef(0);
  const lockedWordRef = useRef<string | null>(null);
  const matchedGraphemesRef = useRef(0);
  const completedRef = useRef(false);
  const sizeRef = useRef({ width: 760, height: 340 });

  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReducedMotion(mq.matches);
    const onChange = () => setReducedMotion(mq.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  // Reset local wave state whenever the active Mini-Game lesson or word list changes.
  useEffect(() => {
    balloonsRef.current = [];
    particlesRef.current = [];
    nextWordIdxRef.current = 0;
    nextIdRef.current = 1;
    spawnAccumMsRef.current = 1800;
    livesRef.current = MAX_LIVES;
    poppedRef.current = 0;
    missedRef.current = 0;
    lockedWordRef.current = null;
    matchedGraphemesRef.current = 0;
    completedRef.current = false;
    setPaused(false);
    setScore(0);
    setLives(MAX_LIVES);
    setPoppedCount(0);
    setMissedCount(0);
    setLockedWord(null);
    setTypedPreview("");
    setInputError(false);
    stageRef.current?.focus();
  }, [activeLessonMeta?.id, miniGameWords]);

  const finishWave = useCallback(() => {
    if (completedRef.current) return;
    completedRef.current = true;
    const storeMetrics = useTypingStore.getState().metrics;
    const baseAcc = storeMetrics.accuracy ?? 100;
    const adjustedAcc = Math.max(0, Math.min(100, baseAcc - missedRef.current * 5));
    const netWpm = storeMetrics.net_wpm ?? 0;
    void completeJungleLesson({
      netWpm,
      accuracy: adjustedAcc,
      errors: storeMetrics.errors + missedRef.current,
      keystrokes: Math.max(storeMetrics.keystrokes, poppedRef.current * 3),
    });
  }, [completeJungleLesson]);

  // Keep canvas scaled cleanly for HiDPI displays on window or pane resize.
  useEffect(() => {
    const host = containerRef.current;
    const canvas = canvasRef.current;
    if (!host || !canvas) return;

    const updateSize = () => {
      const rect = host.getBoundingClientRect();
      const width = Math.max(300, Math.floor(rect.width));
      const height = 340;
      const dpr = window.devicePixelRatio || 1;
      sizeRef.current = { width, height };
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
    };

    updateSize();
    const observer = new ResizeObserver(updateSize);
    observer.observe(host);
    return () => observer.disconnect();
  }, []);

  // Main animation and draw loop: updates falling balloon positions and draws theme-safe HUD shapes.
  useEffect(() => {
    let rafId = 0;
    let lastTs = performance.now();

    const step = (now: number) => {
      const dtMs = Math.min(64, now - lastTs);
      lastTs = now;
      const dtSec = dtMs / 1000;
      const { width, height } = sizeRef.current;

      if (!paused && !lessonCompletion && !completedRef.current) {
        spawnAccumMsRef.current += dtMs;
        const maxActive = 4;
        const spawnInterval = reducedMotion ? 2600 : 2100;

        if (
          spawnAccumMsRef.current >= spawnInterval &&
          balloonsRef.current.length < maxActive &&
          nextWordIdxRef.current < words.length
        ) {
          spawnAccumMsRef.current = 0;
          const word = words[nextWordIdxRef.current++];
          const lane = (nextIdRef.current % 5) * 0.16 + 0.18;
          const baseSpeed = reducedMotion ? 22 : 34 + Math.min(18, poppedRef.current * 1.1);
          balloonsRef.current.push({
            id: nextIdRef.current++,
            word,
            xRatio: Math.min(0.84, Math.max(0.16, lane)),
            yPx: 26,
            speedPxPerSec: baseSpeed,
          });
        }

        const survivors: FallingBalloon[] = [];
        let lostThisFrame = 0;
        for (const b of balloonsRef.current) {
          const nextY = b.yPx + b.speedPxPerSec * dtSec;
          if (nextY >= height - 28) {
            lostThisFrame += 1;
            if (lockedWordRef.current === b.word) {
              lockedWordRef.current = null;
              matchedGraphemesRef.current = 0;
              setLockedWord(null);
              setTypedPreview("");
            }
          } else {
            survivors.push({ ...b, yPx: nextY });
          }
        }
        balloonsRef.current = survivors;

        if (lostThisFrame > 0) {
          missedRef.current += lostThisFrame;
          livesRef.current = Math.max(0, livesRef.current - lostThisFrame);
          setMissedCount(missedRef.current);
          setLives(livesRef.current);
        }

        if (!reducedMotion) {
          particlesRef.current = particlesRef.current
            .map((p) => ({
              ...p,
              x: p.x + p.vx * dtSec,
              y: p.y + p.vy * dtSec,
              life: p.life - dtSec * 2.2,
            }))
            .filter((p) => p.life > 0);
        } else {
          particlesRef.current = [];
        }

        const waveExhausted =
          nextWordIdxRef.current >= words.length && balloonsRef.current.length === 0;
        if (livesRef.current <= 0 || waveExhausted) {
          finishWave();
        }
      }

      const canvas = canvasRef.current;
      const ctx = canvas?.getContext("2d");
      if (canvas && ctx) {
        const dpr = window.devicePixelRatio || 1;
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        ctx.clearRect(0, 0, width, height);

        const isDark = theme === "dark";
        const floorColor = isDark ? "rgba(148, 163, 184, 0.25)" : "rgba(71, 85, 105, 0.2)";
        ctx.strokeStyle = floorColor;
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        ctx.moveTo(16, height - 24);
        ctx.lineTo(width - 16, height - 24);
        ctx.stroke();

        // Draw balloons
        ctx.font = isHindi
          ? '600 18px "Noto Sans Devanagari", system-ui, sans-serif'
          : '600 16px "JetBrains Mono", ui-monospace, monospace';
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";

        for (const b of balloonsRef.current) {
          const cx = b.xRatio * width;
          const cy = b.yPx;
          const isLocked = lockedWordRef.current === b.word;
          const textWidth = Math.max(68, ctx.measureText(b.word).width + 30);
          const pillH = 38;

          // Balloon string
          ctx.strokeStyle = isDark ? "#64748b" : "#94a3b8";
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(cx, cy + pillH / 2);
          ctx.lineTo(cx, cy + pillH / 2 + 12);
          ctx.stroke();

          // Balloon capsule
          ctx.fillStyle = isLocked
            ? isDark
              ? "#451a03"
              : "#fef3c7"
            : isDark
              ? "#1e293b"
              : "#ffffff";
          ctx.strokeStyle = isLocked
            ? isDark
              ? "#f59e0b"
              : "#b45309"
            : isDark
              ? "#475569"
              : "#cbd5e1";
          ctx.lineWidth = isLocked ? 2.5 : 1.5;
          ctx.beginPath();
          ctx.roundRect(cx - textWidth / 2, cy - pillH / 2, textWidth, pillH, 19);
          ctx.fill();
          ctx.stroke();

          ctx.fillStyle = isLocked
            ? isDark
              ? "#fcd34d"
              : "#92400e"
            : isDark
              ? "#f1f5f9"
              : "#0f172a";
          ctx.fillText(b.word, cx, cy + 1);
        }

        // Draw subtle pop sparks when motion is allowed
        for (const p of particlesRef.current) {
          ctx.fillStyle = isDark
            ? `rgba(245, 158, 11, ${p.life.toFixed(2)})`
            : `rgba(180, 83, 9, ${p.life.toFixed(2)})`;
          ctx.beginPath();
          ctx.arc(p.x, p.y, 3, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      rafId = requestAnimationFrame(step);
    };

    rafId = requestAnimationFrame(step);
    return () => cancelAnimationFrame(rafId);
  }, [finishWave, isHindi, lessonCompletion, paused, reducedMotion, theme, words]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    setModifiers({
      capsLock: e.getModifierState("CapsLock"),
      shiftHeld: e.shiftKey,
      altGrHeld: e.getModifierState("AltGraph"),
    });

    if (
      e.key === "Shift" ||
      e.key === "Control" ||
      e.key === "Alt" ||
      e.key === "AltGraph" ||
      e.key === "CapsLock" ||
      e.key === "Meta" ||
      e.metaKey ||
      e.ctrlKey
    ) {
      return;
    }

    if (paused || lessonCompletion || completedRef.current) return;
    e.preventDefault();

    // Pass falling words sorted lowest-first so shared initial letters pop the most urgent balloon.
    const candidates = [...balloonsRef.current]
      .sort((a, b) => b.yPx - a.yPx)
      .map((b) => b.word);

    void resolveGameKeystroke(
      {
        code: e.code,
        shift: e.shiftKey,
        alt_gr: e.getModifierState("AltGraph"),
        ctrl: e.ctrlKey,
        caps_lock: e.getModifierState("CapsLock"),
        timestamp_ms: performance.now(),
      },
      candidates,
    ).then((res) => {
      if (!res) return;
      setInputError(res.is_error);

      if (res.popped) {
        const poppedWord = res.locked_word ?? lockedWordRef.current;
        const idx = balloonsRef.current.findIndex((b) => b.word === poppedWord);
        if (idx >= 0) {
          const hit = balloonsRef.current[idx];
          const cx = hit.xRatio * sizeRef.current.width;
          if (!reducedMotion) {
            for (let k = 0; k < 8; k++) {
              const angle = (Math.PI * 2 * k) / 8;
              particlesRef.current.push({
                x: cx,
                y: hit.yPx,
                vx: Math.cos(angle) * 75,
                vy: Math.sin(angle) * 75,
                life: 1,
              });
            }
          }
          balloonsRef.current.splice(idx, 1);
        }
        poppedRef.current += 1;
        lockedWordRef.current = null;
        matchedGraphemesRef.current = 0;
        setPoppedCount(poppedRef.current);
        setScore((s) => s + 100);
        setLockedWord(null);
        setTypedPreview("");
      } else {
        lockedWordRef.current = res.locked_word;
        matchedGraphemesRef.current = res.matched_graphemes;
        setLockedWord(res.locked_word);
        setTypedPreview(res.typed_preview);
      }
    });
  };

  return (
    <div className="flex w-full flex-col gap-3 rounded-2xl border border-border bg-card p-5 shadow-xs">
      {/* Mini-Game Header Bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-border pb-3">
        <div className="flex items-center gap-2.5">
          <Button type="button" variant="outline" size="sm" onClick={exitActiveLessonToMap}>
            <Map className="h-3.5 w-3.5" aria-hidden="true" />
            {t.jungle.backToMap}
          </Button>
          <div>
            <div className="flex items-center gap-2">
              <Gamepad2 className="h-4 w-4 text-primary" aria-hidden="true" />
              <h2 className="text-sm font-bold text-foreground">
                {activeLessonMeta
                  ? uiLang === "hi"
                    ? activeLessonMeta.title_hi
                    : activeLessonMeta.title_en
                  : t.jungle.game.title}
              </h2>
            </div>
            <p className="text-[11px] text-muted-foreground">{t.jungle.game.instructions}</p>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {reducedMotion && (
            <Badge variant="default">{t.jungle.game.calmMotionNote}</Badge>
          )}
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => {
              setPaused((p) => !p);
              stageRef.current?.focus();
            }}
          >
            {paused ? (
              <Play className="h-3.5 w-3.5" aria-hidden="true" />
            ) : (
              <Pause className="h-3.5 w-3.5" aria-hidden="true" />
            )}
            {paused ? t.jungle.game.resume : t.jungle.game.pause}
          </Button>
          <Button
            type="button"
            variant="secondary"
            size="sm"
            onClick={() => void restartActiveLesson()}
          >
            <RotateCcw className="h-3.5 w-3.5" aria-hidden="true" />
            {t.jungle.game.restart}
          </Button>
          <Button type="button" variant="default" size="sm" onClick={finishWave}>
            <Trophy className="h-3.5 w-3.5" aria-hidden="true" />
            {t.jungle.game.finishWave}
          </Button>
        </div>
      </div>

      {/* Live Game Telemetry */}
      <div className="flex flex-wrap items-center justify-between gap-2 text-xs">
        <div className="flex flex-wrap items-center gap-3">
          <span className="rounded-lg border border-border bg-background/70 px-2.5 py-1 font-mono font-semibold">
            {t.jungle.game.score}: {score}
          </span>
          <span className="rounded-lg border border-border bg-background/70 px-2.5 py-1 font-mono">
            {t.jungle.game.popped}: {poppedCount}/{words.length}
          </span>
          <span className="rounded-lg border border-border bg-background/70 px-2.5 py-1 font-mono">
            {t.jungle.game.missed}: {missedCount}
          </span>
          <span className="rounded-lg border border-border bg-background/70 px-2.5 py-1 font-mono">
            {t.metrics.wpm}: {Math.round(metrics.net_wpm ?? 0)}
          </span>
        </div>

        <div
          className="flex items-center gap-1 rounded-lg border border-border bg-background/70 px-2.5 py-1"
          aria-label={`${t.jungle.game.lives}: ${lives} of ${MAX_LIVES}`}
        >
          <span className="mr-1 font-medium text-muted-foreground">{t.jungle.game.lives}:</span>
          {Array.from({ length: MAX_LIVES }).map((_, idx) => (
            <Heart
              key={idx}
              aria-hidden="true"
              className={cn(
                "h-3.5 w-3.5",
                idx < lives ? "fill-rose-500 text-rose-500" : "text-muted-foreground/30",
              )}
            />
          ))}
        </div>
      </div>

      {/* Interactive Canvas Stage */}
      <div
        ref={stageRef}
        tabIndex={0}
        role="application"
        aria-label={t.jungle.game.title}
        onKeyDown={handleKeyDown}
        onClick={() => stageRef.current?.focus()}
        className="relative overflow-hidden rounded-xl border border-border bg-background/60 focus-visible:ring-2 focus-visible:ring-primary"
      >
        <div ref={containerRef} className="w-full">
          <canvas ref={canvasRef} className="block w-full" />
        </div>
      </div>

      {/* Bottom Locked Target & Input Buffer Readout */}
      <div className="flex flex-wrap items-center justify-between gap-2 text-xs">
        <div className="flex flex-wrap items-center gap-3">
          <span className="text-muted-foreground">
            {t.jungle.game.activeTarget}:{" "}
            <strong
              className={cn(
                "text-foreground",
                isHindi && "font-[family-name:var(--font-devanagari)] text-sm",
              )}
            >
              {lockedWord ?? "--"}
            </strong>
          </span>
          <span className="text-muted-foreground">
            {t.jungle.game.typingBuffer}:{" "}
            <strong
              className={cn(
                inputError ? "text-destructive" : "text-primary",
                isHindi ? "font-[family-name:var(--font-devanagari)] text-sm" : "font-mono",
              )}
            >
              {typedPreview || pendingImePreview || "--"}
            </strong>
          </span>
        </div>
        <span className="text-[11px] text-muted-foreground">{t.jungle.game.switchHint}</span>
      </div>
    </div>
  );
}
