import { createContext, useContext, useEffect, useState } from "react";
import { api, type MarketSummary } from "../api";

export type Connection = "connecting" | "live" | "offline";

export interface MarketState {
  data: MarketSummary | null;
  connection: Connection;
  error: string | null;
}

/**
 * Subscribes to the server's live stream. The server sends the current snapshot as soon as
 * we connect, then a new one whenever the board changes. EventSource reconnects by itself.
 */
export function useMarketStream(): MarketState {
  const [state, setState] = useState<MarketState>({ data: null, connection: "connecting", error: null });

  useEffect(() => {
    const controller = new AbortController();

    // Fetch once as well, so we can show a useful error (e.g. "no data yet") straight away.
    api
      .latest(controller.signal)
      .then((data) => setState((s) => (s.data ? s : { ...s, data, error: null })))
      .catch((e: unknown) => {
        if (controller.signal.aborted) return;
        setState((s) => ({ ...s, error: e instanceof Error ? e.message : String(e) }));
      });

    if (typeof EventSource === "undefined") return () => controller.abort();

    const source = new EventSource(api.streamUrl);
    source.onopen = () => setState((s) => ({ ...s, connection: "live" }));
    source.onerror = () => setState((s) => ({ ...s, connection: "offline" }));
    source.addEventListener("market", (event) => {
      const data = JSON.parse((event as MessageEvent<string>).data) as MarketSummary;
      setState({ data, connection: "live", error: null });
    });

    return () => {
      controller.abort();
      source.close();
    };
  }, []);

  return state;
}

export const MarketContext = createContext<MarketState>({ data: null, connection: "connecting", error: null });

export const useMarket = () => useContext(MarketContext);
