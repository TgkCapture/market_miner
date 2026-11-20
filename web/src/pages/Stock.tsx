import { useEffect, useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api, type History, type SecurityDetail } from "../api";
import { useMarket } from "../hooks/useMarket";
import { useReveal } from "../hooks/useReveal";
import type { Theme } from "../hooks/useTheme";
import { formatAsOf, formatChange, formatKwacha, formatNumber, formatPercent, formatPrice } from "../format";
import { ChangePill } from "../components/Change";
import { PriceChart } from "../components/PriceChart";
import { Skeleton } from "../components/Skeleton";

const RANGES = [
  { label: "1W", days: 7 },
  { label: "1M", days: 30 },
  { label: "3M", days: 90 },
  { label: "1Y", days: 365 },
  { label: "All", days: null },
] as const;

type Range = (typeof RANGES)[number]["label"];

export function Stock({ theme }: { theme: Theme }) {
  const { symbol = "" } = useParams();
  const { data: market } = useMarket();
  const [detail, setDetail] = useState<SecurityDetail | null>(null);
  const [history, setHistory] = useState<History | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [range, setRange] = useState<Range>("All");
  useReveal(detail !== null);

  // The live snapshot keeps the headline price current without refetching.
  const liveQuote = market?.quotes.find((q) => q.symbol === symbol.toUpperCase()) ?? null;
  const quote = liveQuote ?? detail?.quote ?? null;

  useEffect(() => {
    const controller = new AbortController();
    setDetail(null);
    setError(null);
    api.security(symbol, controller.signal).then(setDetail).catch((e: Error) => {
      if (!controller.signal.aborted) setError(e.message);
    });
    return () => controller.abort();
  }, [symbol]);

  // Refetch history when the range changes or a new snapshot arrives.
  useEffect(() => {
    const controller = new AbortController();
    const days = RANGES.find((r) => r.label === range)?.days;
    const from = days ? new Date(Date.now() - days * 86_400_000) : undefined;
    api.history(symbol, from, controller.signal).then(setHistory).catch(() => {});
    return () => controller.abort();
  }, [symbol, range, market?.as_of]);

  const rangeChange = useMemo(() => {
    const pts = history?.points ?? [];
    if (pts.length < 2) return null;
    const first = pts[0].close;
    const last = pts[pts.length - 1].close;
    return { abs: last - first, pct: first ? ((last - first) / first) * 100 : 0 };
  }, [history]);

  if (error) {
    return (
      <main className="container page">
        <Link to="/" className="back">← All securities</Link>
        <div className="card notice" role="alert"><strong>{symbol.toUpperCase()}</strong> <span className="muted">{error}</span></div>
      </main>
    );
  }

  return (
    <main className="container page">
      <Link to="/" className="back">← All securities</Link>

      <section className="stock-head">
        <div>
          <p className="eyebrow">{detail?.isin ?? " "}</p>
          <h1 className="stock-symbol">{symbol.toUpperCase()}</h1>
        </div>
        <div className="stock-price">
          {quote ? (
            <>
              <span className="price-big">{formatPrice(quote.close)}</span>
              <div className="price-change">
                <span className={`text-${quote.change > 0 ? "up" : quote.change < 0 ? "down" : "flat"}`}>{formatChange(quote.change)}</span>
                <ChangePill pct={quote.change_pct} size="lg" />
              </div>
            </>
          ) : (
            <Skeleton width={180} height={44} />
          )}
        </div>
      </section>

      <section className="card chart-card reveal">
        <div className="chart-toolbar">
          <div className="range-tabs" role="tablist" aria-label="Chart range">
            {RANGES.map((r) => (
              <button key={r.label} type="button" role="tab" aria-selected={range === r.label}
                className={`tab ${range === r.label ? "tab-active" : ""}`} onClick={() => setRange(r.label)}>
                {r.label}
              </button>
            ))}
          </div>
          {rangeChange && (
            <span className="muted">
              {range === "All" ? "Since tracking began" : `Past ${range}`}:{" "}
              <span className={`text-${rangeChange.abs > 0 ? "up" : rangeChange.abs < 0 ? "down" : "flat"}`}>
                {formatChange(rangeChange.abs)} ({formatPercent(rangeChange.pct)})
              </span>
            </span>
          )}
        </div>
        {history && history.points.length > 1 ? (
          <PriceChart points={history.points} theme={theme} />
        ) : history ? (
          <div className="chart chart-empty muted">
            Not enough history yet. The chart fills in as new prices are recorded.
          </div>
        ) : (
          <div className="chart"><Skeleton height="100%" radius={12} /></div>
        )}
      </section>

      <section className="facts">
        {[
          ["Previous close", quote && formatPrice(quote.open)],
          ["Close", quote && formatPrice(quote.close)],
          ["Volume", quote && formatNumber(quote.volume)],
          ["Turnover", quote && formatKwacha(quote.turnover)],
          ["ISIN", detail?.isin ?? "—"],
          ["Tracked since", detail?.first_seen ? formatAsOf(detail.first_seen) : "—"],
          ["Data points", detail ? formatNumber(detail.data_points) : null],
          ["Last update", detail?.last_seen ? formatAsOf(detail.last_seen) : "—"],
        ].map(([label, value]) => (
          <div key={label} className="card fact reveal">
            <span className="stat-label">{label}</span>
            <span className="fact-value">{value ?? <Skeleton width="60%" />}</span>
          </div>
        ))}
      </section>
    </main>
  );
}
