import type { MarketSummary } from "../api";
import { formatCompact, formatKwacha } from "../format";

/** Headline numbers for the board plus a breadth bar (advancers / unchanged / decliners). */
export function StatStrip({ data }: { data: MarketSummary }) {
  const total = Math.max(1, data.advancers + data.decliners + data.unchanged);
  const pct = (n: number) => `${(n / total) * 100}%`;

  return (
    <section className="stats" aria-label="Market summary">
      <div className="card stat reveal">
        <span className="stat-label">Turnover</span>
        <span className="stat-value">{formatKwacha(data.total_turnover)}</span>
      </div>
      <div className="card stat reveal">
        <span className="stat-label">Shares traded</span>
        <span className="stat-value">{formatCompact(data.total_volume)}</span>
      </div>
      <div className="card stat reveal">
        <span className="stat-label">Listed</span>
        <span className="stat-value">{data.quotes.length}</span>
      </div>
      <div className="card stat stat-breadth reveal">
        <span className="stat-label">Breadth</span>
        <div className="breadth" role="img"
          aria-label={`${data.advancers} up, ${data.unchanged} unchanged, ${data.decliners} down`}>
          <span className="breadth-up" style={{ width: pct(data.advancers) }} />
          <span className="breadth-flat" style={{ width: pct(data.unchanged) }} />
          <span className="breadth-down" style={{ width: pct(data.decliners) }} />
        </div>
        <div className="breadth-legend">
          <span className="text-up">▲ {data.advancers}</span>
          <span className="muted">• {data.unchanged}</span>
          <span className="text-down">▼ {data.decliners}</span>
        </div>
      </div>
    </section>
  );
}
