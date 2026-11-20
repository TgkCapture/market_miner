import { Link, Route, Routes, useLocation } from "react-router-dom";
import { Header } from "./components/Header";
import { MarketContext, useMarketStream } from "./hooks/useMarket";
import { useScrollToTopOnNavigate, useSmoothScroll } from "./hooks/useSmoothScroll";
import { useTheme } from "./hooks/useTheme";
import { Overview } from "./pages/Overview";
import { Stock } from "./pages/Stock";

export function App() {
  const market = useMarketStream();
  const { theme, toggle } = useTheme();
  const location = useLocation();
  useSmoothScroll();
  useScrollToTopOnNavigate();

  return (
    <MarketContext.Provider value={market}>
      <div className="backdrop" aria-hidden="true" />
      <Header theme={theme} onToggleTheme={toggle} />
      {/* Keyed on the path so each page plays its enter transition. */}
      <div key={location.pathname} className="page-transition">
        <Routes location={location}>
          <Route path="/" element={<Overview />} />
          <Route path="/stock/:symbol" element={<Stock theme={theme} />} />
          <Route path="*" element={
            <main className="container page">
              <h1>Page not found</h1>
              <Link to="/" className="back">← Back to the market</Link>
            </main>
          } />
        </Routes>
      </div>
      <footer className="container footer muted">
        Data from the Malawi Stock Exchange website. Not investment advice.
      </footer>
    </MarketContext.Provider>
  );
}
