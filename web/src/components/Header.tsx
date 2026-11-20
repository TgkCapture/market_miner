import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { useMarket } from "../hooks/useMarket";
import { formatAsOf } from "../format";
import type { Theme } from "../hooks/useTheme";

export function Header({ theme, onToggleTheme }: { theme: Theme; onToggleTheme: () => void }) {
  const { data, connection } = useMarket();
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const status = data?.status ?? "unknown";

  return (
    <header className={`topbar ${scrolled ? "topbar-scrolled" : ""}`}>
      <div className="container topbar-inner">
        <Link to="/" className="brand" aria-label="Market Miner home">
          <svg viewBox="0 0 32 32" width="26" height="26" aria-hidden="true">
            <rect width="32" height="32" rx="8" className="brand-bg" />
            <path d="M6 22 L12 15 L17 19 L26 9" fill="none" className="brand-line" strokeWidth="3"
              strokeLinecap="round" strokeLinejoin="round" />
          </svg>
          <span>Market Miner</span>
        </Link>

        <div className="topbar-meta">
          {data && (
            <span className="muted hide-sm" title="When the exchange published these prices">
              {formatAsOf(data.as_of)}
            </span>
          )}
          {status !== "unknown" && (
            <span className={`status status-${status}`}>Market {status}</span>
          )}
          <span className={`live live-${connection}`} title={`Live updates: ${connection}`}>
            <span className="live-dot" />
            <span className="hide-sm">{connection === "live" ? "Live" : connection === "connecting" ? "Connecting" : "Reconnecting"}</span>
          </span>
          <button type="button" className="icon-btn" onClick={onToggleTheme}
            aria-label={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}>
            {theme === "dark" ? (
              <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true"><circle cx="12" cy="12" r="4.5" fill="currentColor" /><g stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" /></g></svg>
            ) : (
              <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true"><path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5Z" fill="currentColor" /></svg>
            )}
          </button>
        </div>
      </div>
    </header>
  );
}
