import { useEffect, useRef } from "react";
import {
  CategoryScale,
  Chart,
  Filler,
  Legend,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
  Tooltip,
} from "chart.js";
import type { SessionRecord } from "@/lib/bindings";
import { getDictionary } from "@/i18n/useTranslation";
import { useTypingStore } from "@/stores/useTypingStore";

Chart.register(
  LineController,
  LineElement,
  PointElement,
  LinearScale,
  CategoryScale,
  Tooltip,
  Legend,
  Filler,
);

export function WpmTrendChart({ sessions }: { sessions: SessionRecord[] }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartRef = useRef<Chart | null>(null);
  const theme = useTypingStore((s) => s.theme);
  const uiLang = useTypingStore((s) => s.uiLang);
  const t = getDictionary(uiLang);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    if (chartRef.current) {
      chartRef.current.destroy();
      chartRef.current = null;
    }

    if (sessions.length === 0) return;

    const ordered = [...sessions].reverse();
    const isDark = theme === "dark";
    const gridColor = isDark ? "rgba(148, 163, 184, 0.14)" : "rgba(15, 23, 42, 0.08)";
    const textColor = isDark ? "#94a3b8" : "#475569";

    chartRef.current = new Chart(canvas, {
      type: "line",
      data: {
        labels: ordered.map((_, idx) => `#${idx + 1}`),
        datasets: [
          {
            label: t.metrics.netWpm,
            data: ordered.map((s) => Math.round(s.net_wpm ?? 0)),
            borderColor: "#f59e0b",
            backgroundColor: "rgba(245, 158, 11, 0.14)",
            fill: true,
            tension: 0.3,
            pointRadius: 3,
            yAxisID: "yWpm",
          },
          {
            label: t.metrics.grossWpm,
            data: ordered.map((s) => Math.round(s.gross_wpm ?? 0)),
            borderColor: "#64748b",
            borderDash: [4, 4],
            tension: 0.3,
            pointRadius: 2,
            yAxisID: "yWpm",
          },
          {
            label: t.metrics.accuracy,
            data: ordered.map((s) => Number((s.accuracy ?? 100).toFixed(1))),
            borderColor: "#10b981",
            tension: 0.3,
            pointRadius: 3,
            yAxisID: "yAcc",
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { mode: "index", intersect: false },
        plugins: {
          legend: {
            labels: { color: textColor, boxWidth: 12, font: { size: 11 } },
          },
        },
        scales: {
          x: {
            grid: { color: gridColor },
            ticks: { color: textColor, font: { size: 11 } },
          },
          yWpm: {
            type: "linear",
            position: "left",
            beginAtZero: true,
            grid: { color: gridColor },
            ticks: { color: textColor, font: { size: 11 } },
          },
          yAcc: {
            type: "linear",
            position: "right",
            min: 50,
            max: 100,
            grid: { drawOnChartArea: false },
            ticks: {
              color: "#10b981",
              font: { size: 11 },
              callback: (v) => `${v}%`,
            },
          },
        },
      },
    });

    return () => {
      chartRef.current?.destroy();
      chartRef.current = null;
    };
  }, [sessions, theme, t.metrics.accuracy, t.metrics.grossWpm, t.metrics.netWpm]);

  if (sessions.length === 0) {
    return (
      <div className="flex h-60 items-center justify-center rounded-xl border border-dashed border-border text-xs text-muted-foreground">
        {t.analytics.noData}
      </div>
    );
  }

  return (
    <div className="relative h-64 w-full">
      <canvas ref={canvasRef} />
    </div>
  );
}
