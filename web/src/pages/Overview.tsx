import { useRef } from "react";
import { useMarket } from "../hooks/useMarket";
import { useReveal } from "../hooks/useReveal";
import { scrollToElement } from "../hooks/useSmoothScroll";
import { formatAsOf, timeAgo } from "../format";
import { StatStrip } from "../components/StatStrip";
import { Movers } from "../components/Movers";
import { MarketTable } from "../components/MarketTable";
import { OverviewSkeleton } from "../components/Skeleton";

export function Overview() {
  const { data, error } = useMarket();
  const tableRef = useRef<HTMLDivElement>(null);
  useReveal(data !== null);

  return (
    <main className="container page">
      <section className="hero">
        <p className="eyebrow">Malawi Stock Exchange · Mainboard</p>
        <h1>The market, at a glance.</h1>
        <p className="hero-sub muted">
          {data
            ? <>Prices as at <strong>{formatAsOf(data.as_of)}</strong> ({timeAgo(data.as_of)}). Updates arrive live.</>
            : "Live prices, movers and history for every listed company."}
        </p>
        {data && (
          <button type="button" className="btn-ghost" onClick={() => tableRef.current && scrollToElement(tableRef.current)}>
            View all {data.quotes.length} securities ↓
          </button>
        )}
      </section>

      {!data && error && (
        <div className="card notice" role="status">
          <strong>No data yet.</strong> <span className="muted">{error}</span>
        </div>
      )}

      {!data && !error && <OverviewSkeleton />}

      {data && (
        <>
          <StatStrip data={data} />
          <Movers quotes={data.quotes} />
          <div ref={tableRef}>
            <MarketTable quotes={data.quotes} />
          </div>
        </>
      )}
    </main>
  );
}
