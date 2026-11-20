// Types mirror the JSON returned by the Rust API (src/api).

export type MarketStatus = "open" | "closed" | "unknown";

export interface Quote {
  symbol: string;
  isin: string | null;
  open: number;
  close: number;
  change: number;
  change_pct: number;
  volume: number;
  turnover: number;
}

export interface MarketSummary {
  as_of: string;
  status: MarketStatus;
  quotes: Quote[];
  total_turnover: number;
  total_volume: number;
  advancers: number;
  decliners: number;
  unchanged: number;
}

export interface Security {
  symbol: string;
  isin: string | null;
  first_seen: string | null;
  last_seen: string | null;
  data_points: number;
}

export interface SecurityDetail extends Security {
  quote: Quote | null;
}

export interface HistoryPoint {
  as_of: string;
  open: number;
  close: number;
  change: number;
  change_pct: number;
  volume: number;
  turnover: number;
}

export interface History {
  symbol: string;
  points: HistoryPoint[];
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  const res = await fetch(path, { signal, headers: { Accept: "application/json" } });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new ApiError(res.status, body?.error ?? res.statusText);
  }
  return res.json() as Promise<T>;
}

export const api = {
  latest: (signal?: AbortSignal) => getJson<MarketSummary>("/api/market/latest", signal),
  security: (symbol: string, signal?: AbortSignal) =>
    getJson<SecurityDetail>(`/api/securities/${encodeURIComponent(symbol)}`, signal),
  history: (symbol: string, from?: Date, signal?: AbortSignal) => {
    const params = new URLSearchParams({ limit: "5000" });
    if (from) params.set("from", from.toISOString());
    return getJson<History>(`/api/securities/${encodeURIComponent(symbol)}/history?${params}`, signal);
  },
  streamUrl: "/api/stream",
};
