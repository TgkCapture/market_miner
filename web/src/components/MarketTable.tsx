import { useEffect, useMemo, useRef, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import type { Quote } from "../api";
import { formatCompact, formatKwacha, formatPrice } from "../format";
import { ChangePill, ChangeText } from "./Change";

type SortKey = "symbol" | "close" | "change_pct" | "volume" | "turnover";
type Sort = { key: SortKey; dir: "asc" | "desc" };

const COLUMNS: { key: SortKey; label: string; numeric: boolean }[] = [
  { key: "symbol", label: "Symbol", numeric: false },
  { key: "close", label: "Price", numeric: true },
  { key: "change_pct", label: "Change", numeric: true },
  { key: "volume", label: "Volume", numeric: true },
  { key: "turnover", label: "Turnover", numeric: true },
];

export function sortQuotes(quotes: Quote[], sort: Sort): Quote[] {
  const sign = sort.dir === "asc" ? 1 : -1;
  return [...quotes].sort((a, b) => {
    const av = a[sort.key];
    const bv = b[sort.key];
    const cmp = typeof av === "string" ? av.localeCompare(bv as string) : (av as number) - (bv as number);
    return cmp * sign || a.symbol.localeCompare(b.symbol);
  });
}

/** Remembers each symbol's last price and returns which ones just moved, for a brief flash. */
function useFlashes(quotes: Quote[]) {
  const previous = useRef(new Map<string, number>());
  const [flashes, setFlashes] = useState<Record<string, "up" | "down">>({});

  useEffect(() => {
    const next: Record<string, "up" | "down"> = {};
    for (const q of quotes) {
      const before = previous.current.get(q.symbol);
      if (before !== undefined && before !== q.close) next[q.symbol] = q.close > before ? "up" : "down";
      previous.current.set(q.symbol, q.close);
    }
    if (Object.keys(next).length === 0) return;
    setFlashes(next);
    const t = window.setTimeout(() => setFlashes({}), 1600);
    return () => window.clearTimeout(t);
  }, [quotes]);

  return flashes;
}

export function MarketTable({ quotes }: { quotes: Quote[] }) {
  const [sort, setSort] = useState<Sort>({ key: "turnover", dir: "desc" });
  const [query, setQuery] = useState("");
  const navigate = useNavigate();
  const flashes = useFlashes(quotes);

  const rows = useMemo(() => {
    const q = query.trim().toUpperCase();
    const filtered = q ? quotes.filter((x) => x.symbol.includes(q) || x.isin?.includes(q)) : quotes;
    return sortQuotes(filtered, sort);
  }, [quotes, query, sort]);

  const onSort = (key: SortKey) =>
    setSort((s) => (s.key === key ? { key, dir: s.dir === "asc" ? "desc" : "asc" } : { key, dir: key === "symbol" ? "asc" : "desc" }));

  return (
    <section className="card table-card reveal" aria-label="All securities">
      <div className="table-toolbar">
        <h2 className="section-title">Mainboard</h2>
        <label className="search">
          <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><circle cx="11" cy="11" r="7" fill="none" stroke="currentColor" strokeWidth="2" /><path d="m20 20-4-4" stroke="currentColor" strokeWidth="2" strokeLinecap="round" /></svg>
          <input type="search" placeholder="Search symbol" value={query}
            onChange={(e) => setQuery(e.target.value)} aria-label="Search by symbol" />
        </label>
      </div>

      <div className="table-scroll">
        <table className="board">
          <thead>
            <tr>
              {COLUMNS.map((c) => {
                const active = sort.key === c.key;
                return (
                  <th key={c.key} className={`${c.numeric ? "num" : ""} col-${c.key}`}
                    aria-sort={active ? (sort.dir === "asc" ? "ascending" : "descending") : "none"}>
                    <button type="button" className={`sort ${active ? "sort-active" : ""}`} onClick={() => onSort(c.key)}>
                      {c.label}
                      <span className="sort-arrow" aria-hidden="true">{active ? (sort.dir === "asc" ? "↑" : "↓") : "↕"}</span>
                    </button>
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody>
            {rows.map((q) => (
              <tr key={q.symbol} className={flashes[q.symbol] ? `flash-${flashes[q.symbol]}` : ""}
                onClick={() => navigate(`/stock/${q.symbol}`)}>
                <td>
                  <Link to={`/stock/${q.symbol}`} className="symbol" onClick={(e) => e.stopPropagation()}>{q.symbol}</Link>
                  {q.isin && <span className="isin">{q.isin}</span>}
                </td>
                <td className="num strong">{formatPrice(q.close)}</td>
                <td className="num">
                  <div className="change-cell">
                    <ChangeText value={q.change} />
                    <ChangePill pct={q.change_pct} />
                  </div>
                </td>
                <td className="num col-volume">{formatCompact(q.volume)}</td>
                <td className="num">{formatKwacha(q.turnover)}</td>
              </tr>
            ))}
            {rows.length === 0 && (
              <tr><td colSpan={5} className="empty muted">No symbols match “{query}”.</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
