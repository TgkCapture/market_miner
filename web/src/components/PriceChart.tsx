import { useEffect, useRef } from "react";
import { ColorType, CrosshairMode, createChart, type IChartApi, type ISeriesApi, type UTCTimestamp } from "lightweight-charts";
import type { HistoryPoint } from "../api";
import type { Theme } from "../hooks/useTheme";

function cssVar(name: string) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

/** "#34d399" + 0.3 → "rgba(52, 211, 153, 0.3)". The canvas chart needs plain colour strings. */
export function withAlpha(hex: string, alpha: number): string {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex);
  if (!m) return hex;
  const n = parseInt(m[1], 16);
  return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${alpha})`;
}

/** Area chart of closing prices. Colour follows the direction over the shown range. */
export function PriceChart({ points, theme }: { points: HistoryPoint[]; theme: Theme }) {
  const container = useRef<HTMLDivElement>(null);
  const chart = useRef<IChartApi | null>(null);
  const series = useRef<ISeriesApi<"Area"> | null>(null);

  useEffect(() => {
    if (!container.current) return;
    chart.current = createChart(container.current, {
      autoSize: true,
      crosshair: { mode: CrosshairMode.Magnet },
      rightPriceScale: { borderVisible: false, scaleMargins: { top: 0.15, bottom: 0.1 } },
      timeScale: { borderVisible: false, timeVisible: true, secondsVisible: false },
      handleScroll: { mouseWheel: false, pressedMouseMove: true, horzTouchDrag: true, vertTouchDrag: false },
      handleScale: { mouseWheel: false, pinch: true, axisPressedMouseMove: true },
    });
    series.current = chart.current.addAreaSeries({ lineWidth: 2, priceLineVisible: false });
    return () => {
      chart.current?.remove();
      chart.current = null;
      series.current = null;
    };
  }, []);

  // Colours come from CSS variables so the chart matches the theme.
  useEffect(() => {
    if (!chart.current || !series.current) return;
    const first = points[0]?.close ?? 0;
    const last = points[points.length - 1]?.close ?? 0;
    const color = cssVar(last < first ? "--down" : "--up");
    chart.current.applyOptions({
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: cssVar("--text-muted"),
        fontFamily: cssVar("--font"),
      },
      grid: { vertLines: { visible: false }, horzLines: { color: cssVar("--grid") } },
    });
    series.current.applyOptions({
      lineColor: color,
      topColor: withAlpha(color, 0.32),
      bottomColor: withAlpha(color, 0),
    });
  }, [points, theme]);

  useEffect(() => {
    series.current?.setData(
      points.map((p) => ({ time: (new Date(p.as_of).getTime() / 1000) as UTCTimestamp, value: p.close })),
    );
    chart.current?.timeScale().fitContent();
  }, [points]);

  return <div ref={container} className="chart" />;
}
