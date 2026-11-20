import { Link } from "react-router-dom";
import type { Quote } from "../api";
import { formatKwacha, formatPrice } from "../format";
import { ChangePill } from "./Change";

export function topMovers(quotes: Quote[], limit = 5) {
  const gainers = quotes.filter((q) => q.change > 0).sort((a, b) => b.change_pct - a.change_pct).slice(0, limit);
  const losers = quotes.filter((q) => q.change < 0).sort((a, b) => a.change_pct - b.change_pct).slice(0, limit);
  const mostTraded = quotes.filter((q) => q.turnover > 0).sort((a, b) => b.turnover - a.turnover).slice(0, limit);
  return { gainers, losers, mostTraded };
}

function MoverList({ title, quotes, empty, metric }: {
  title: string;
  quotes: Quote[];
  empty: string;
  metric: "change" | "turnover";
}) {
  return (
    <div className="card movers-card reveal">
      <h3 className="card-title">{title}</h3>
      {quotes.length === 0 ? (
        <p className="muted empty">{empty}</p>
      ) : (
        <ol className="mover-list">
          {quotes.map((q) => (
            <li key={q.symbol}>
              <Link to={`/stock/${q.symbol}`} className="mover-row">
                <span className="symbol">{q.symbol}</span>
                <span className="mover-price">{formatPrice(q.close)}</span>
                {metric === "change" ? <ChangePill pct={q.change_pct} /> : (
                  <span className="mover-metric">{formatKwacha(q.turnover)}</span>
                )}
              </Link>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

export function Movers({ quotes }: { quotes: Quote[] }) {
  const { gainers, losers, mostTraded } = topMovers(quotes);
  return (
    <section className="movers" aria-label="Top movers">
      <MoverList title="Top gainers" quotes={gainers} metric="change" empty="No gainers this session." />
      <MoverList title="Top losers" quotes={losers} metric="change" empty="No losers this session." />
      <MoverList title="Most traded" quotes={mostTraded} metric="turnover" empty="No trades yet." />
    </section>
  );
}
