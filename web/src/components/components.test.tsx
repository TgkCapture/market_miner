import { describe, expect, it } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import type { MarketSummary, Quote } from "../api";
import { MarketTable, sortQuotes } from "./MarketTable";
import { Movers, topMovers } from "./Movers";
import { StatStrip } from "./StatStrip";

const q = (symbol: string, open: number, close: number, turnover: number): Quote => ({
  symbol,
  isin: `MW${symbol}`,
  open,
  close,
  change: close - open,
  change_pct: ((close - open) / open) * 100,
  volume: turnover / close,
  turnover,
});

const QUOTES = [q("CHL", 193.04, 194.91, 2_938_484), q("FDHB", 447.85, 447.07, 2_022_471_000), q("PCL", 3300, 3300, 0)];

const symbolsInTable = () =>
  screen.getAllByRole("row").slice(1).map((r) => within(r).getAllByRole("cell")[0].querySelector(".symbol")?.textContent);

describe("MarketTable", () => {
  it("sorts by turnover by default and toggles on header click", () => {
    render(<MemoryRouter><MarketTable quotes={QUOTES} /></MemoryRouter>);
    expect(symbolsInTable()).toEqual(["FDHB", "CHL", "PCL"]);

    fireEvent.click(screen.getByRole("button", { name: /symbol/i }));
    expect(symbolsInTable()).toEqual(["CHL", "FDHB", "PCL"]);

    fireEvent.click(screen.getByRole("button", { name: /symbol/i }));
    expect(symbolsInTable()).toEqual(["PCL", "FDHB", "CHL"]);
  });

  it("filters by search text", () => {
    render(<MemoryRouter><MarketTable quotes={QUOTES} /></MemoryRouter>);
    fireEvent.change(screen.getByRole("searchbox"), { target: { value: "ch" } });
    expect(symbolsInTable()).toEqual(["CHL"]);

    fireEvent.change(screen.getByRole("searchbox"), { target: { value: "zzz" } });
    expect(screen.getByText(/No symbols match/)).toBeInTheDocument();
  });

  it("sortQuotes breaks ties by symbol", () => {
    const tied = [q("B", 1, 1, 5), q("A", 1, 1, 5)];
    expect(sortQuotes(tied, { key: "turnover", dir: "desc" }).map((x) => x.symbol)).toEqual(["A", "B"]);
  });
});

describe("Movers", () => {
  it("picks gainers, losers and most traded", () => {
    const m = topMovers(QUOTES);
    expect(m.gainers.map((x) => x.symbol)).toEqual(["CHL"]);
    expect(m.losers.map((x) => x.symbol)).toEqual(["FDHB"]);
    expect(m.mostTraded.map((x) => x.symbol)).toEqual(["FDHB", "CHL"]);
  });

  it("renders links to each stock", () => {
    render(<MemoryRouter><Movers quotes={QUOTES} /></MemoryRouter>);
    const links = screen.getAllByRole("link", { name: /CHL/ });
    expect(links[0]).toHaveAttribute("href", "/stock/CHL");
  });
});

describe("StatStrip", () => {
  it("shows totals and breadth", () => {
    const data: MarketSummary = {
      as_of: "2026-11-20T13:00:00Z",
      status: "closed",
      quotes: QUOTES,
      total_turnover: 2_025_409_484,
      total_volume: 1000,
      advancers: 1,
      decliners: 1,
      unchanged: 1,
    };
    render(<StatStrip data={data} />);
    expect(screen.getByText("MK 2.03B")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "1 up, 1 unchanged, 1 down" })).toBeInTheDocument();
  });
});
