//! Core market types shared by the scraper, database and API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    Open,
    Closed,
    /// Status wasn't shown on the page, or the snapshot was loaded from the database.
    Unknown,
}

/// One security's prices in a snapshot.
///
/// On the MSE mainboard the "open" price is the previous session's close, so the day's
/// change is `close - open`. The site's own "% Change" column is not reliable (it often
/// reads 0), so we always calculate it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    pub isin: Option<String>,
    pub open: f64,
    pub close: f64,
    pub change: f64,
    pub change_pct: f64,
    pub volume: f64,
    pub turnover: f64,
}

impl Quote {
    pub fn new(
        symbol: impl Into<String>,
        isin: Option<String>,
        open: f64,
        close: f64,
        volume: f64,
        turnover: f64,
    ) -> Self {
        let change = round(close - open, 4);
        let change_pct = if open == 0.0 {
            0.0
        } else {
            round(change / open * 100.0, 4)
        };
        Self {
            symbol: symbol.into(),
            isin,
            open,
            close,
            change,
            change_pct,
            volume,
            turnover,
        }
    }
}

/// The whole board at one point in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketSnapshot {
    /// When the exchange says these prices are from ("Stats as at ...").
    pub as_of: DateTime<Utc>,
    pub status: MarketStatus,
    pub quotes: Vec<Quote>,
}

impl MarketSnapshot {
    pub fn total_turnover(&self) -> f64 {
        self.quotes.iter().map(|q| q.turnover).sum()
    }

    pub fn total_volume(&self) -> f64 {
        self.quotes.iter().map(|q| q.volume).sum()
    }

    pub fn movers(&self, limit: usize) -> Movers {
        let mut gainers: Vec<Quote> = self
            .quotes
            .iter()
            .filter(|q| q.change > 0.0)
            .cloned()
            .collect();
        gainers.sort_by(|a, b| b.change_pct.total_cmp(&a.change_pct));
        gainers.truncate(limit);

        let mut losers: Vec<Quote> = self
            .quotes
            .iter()
            .filter(|q| q.change < 0.0)
            .cloned()
            .collect();
        losers.sort_by(|a, b| a.change_pct.total_cmp(&b.change_pct));
        losers.truncate(limit);

        let mut most_traded: Vec<Quote> = self
            .quotes
            .iter()
            .filter(|q| q.turnover > 0.0)
            .cloned()
            .collect();
        most_traded.sort_by(|a, b| b.turnover.total_cmp(&a.turnover));
        most_traded.truncate(limit);

        Movers {
            gainers,
            losers,
            most_traded,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Movers {
    pub gainers: Vec<Quote>,
    pub losers: Vec<Quote>,
    pub most_traded: Vec<Quote>,
}

fn round(value: f64, places: i32) -> f64 {
    let factor = 10f64.powi(places);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(symbol: &str, open: f64, close: f64, turnover: f64) -> Quote {
        Quote::new(symbol, None, open, close, 1.0, turnover)
    }

    #[test]
    fn quote_calculates_change() {
        let quote = q("CHL", 193.04, 194.91, 0.0);
        assert_eq!(quote.change, 1.87);
        assert_eq!(quote.change_pct, 0.9687);
    }

    #[test]
    fn quote_with_zero_open_has_no_percentage() {
        assert_eq!(q("NEW", 0.0, 10.0, 0.0).change_pct, 0.0);
    }

    #[test]
    fn movers_are_sorted_and_limited() {
        let snap = MarketSnapshot {
            as_of: Utc::now(),
            status: MarketStatus::Closed,
            quotes: vec![
                q("A", 100.0, 110.0, 5.0),  // +10%
                q("B", 100.0, 101.0, 50.0), // +1%
                q("C", 100.0, 90.0, 0.0),   // -10%
                q("D", 100.0, 99.0, 20.0),  // -1%
                q("E", 100.0, 100.0, 1.0),  // flat
            ],
        };
        let m = snap.movers(1);
        assert_eq!(
            m.gainers
                .iter()
                .map(|q| q.symbol.as_str())
                .collect::<Vec<_>>(),
            ["A"]
        );
        assert_eq!(
            m.losers
                .iter()
                .map(|q| q.symbol.as_str())
                .collect::<Vec<_>>(),
            ["C"]
        );
        assert_eq!(
            m.most_traded
                .iter()
                .map(|q| q.symbol.as_str())
                .collect::<Vec<_>>(),
            ["B"]
        );

        let all = snap.movers(10);
        assert_eq!(all.gainers.len(), 2);
        assert_eq!(all.losers.len(), 2);
        assert_eq!(
            all.most_traded.len(),
            4,
            "zero-turnover securities are excluded"
        );
        assert_eq!(snap.total_turnover(), 76.0);
    }
}
