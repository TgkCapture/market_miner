//! Postgres access. All queries live here.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::market::{MarketSnapshot, MarketStatus, Quote};

pub async fn connect(database_url: &str) -> sqlx::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Saves a snapshot. Rows that already exist for the same security and `as_of` are left
/// alone, so saving the same snapshot twice is harmless. Returns how many new quotes were stored.
pub async fn save_snapshot(pool: &PgPool, snapshot: &MarketSnapshot) -> sqlx::Result<u64> {
    let mut tx = pool.begin().await?;
    let mut inserted = 0;
    for quote in &snapshot.quotes {
        let security_id: i64 = sqlx::query_scalar(
            "INSERT INTO securities (symbol, isin) VALUES ($1, $2)
             ON CONFLICT (symbol) DO UPDATE SET isin = COALESCE(EXCLUDED.isin, securities.isin)
             RETURNING id",
        )
        .bind(&quote.symbol)
        .bind(&quote.isin)
        .fetch_one(&mut *tx)
        .await?;

        inserted += sqlx::query(
            "INSERT INTO quotes (security_id, as_of, open_price, close_price, volume, turnover)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (security_id, as_of) DO NOTHING",
        )
        .bind(security_id)
        .bind(snapshot.as_of)
        .bind(quote.open)
        .bind(quote.close)
        .bind(quote.volume)
        .bind(quote.turnover)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    }
    tx.commit().await?;
    Ok(inserted)
}

#[derive(sqlx::FromRow)]
struct QuoteRow {
    symbol: String,
    isin: Option<String>,
    as_of: DateTime<Utc>,
    open_price: f64,
    close_price: f64,
    volume: f64,
    turnover: f64,
}

impl QuoteRow {
    fn into_quote(self) -> Quote {
        Quote::new(
            self.symbol,
            self.isin,
            self.open_price,
            self.close_price,
            self.volume,
            self.turnover,
        )
    }
}

/// Rebuilds the most recent snapshot: each security's latest quote. Used at startup so the
/// API has data before the first scrape finishes.
pub async fn latest_snapshot(pool: &PgPool) -> sqlx::Result<Option<MarketSnapshot>> {
    let rows: Vec<QuoteRow> = sqlx::query_as(
        "SELECT DISTINCT ON (s.id) s.symbol, s.isin, q.as_of, q.open_price, q.close_price, q.volume, q.turnover
         FROM securities s JOIN quotes q ON q.security_id = s.id
         ORDER BY s.id, q.as_of DESC",
    )
    .fetch_all(pool)
    .await?;

    let Some(as_of) = rows.iter().map(|r| r.as_of).max() else {
        return Ok(None);
    };
    let mut quotes: Vec<Quote> = rows.into_iter().map(QuoteRow::into_quote).collect();
    quotes.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    Ok(Some(MarketSnapshot {
        as_of,
        status: MarketStatus::Unknown,
        quotes,
    }))
}

#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct Security {
    pub symbol: String,
    pub isin: Option<String>,
    pub first_seen: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
    pub data_points: i64,
}

pub async fn list_securities(pool: &PgPool) -> sqlx::Result<Vec<Security>> {
    sqlx::query_as(
        "SELECT s.symbol, s.isin, MIN(q.as_of) AS first_seen, MAX(q.as_of) AS last_seen,
                COUNT(q.id) AS data_points
         FROM securities s LEFT JOIN quotes q ON q.security_id = s.id
         GROUP BY s.id ORDER BY s.symbol",
    )
    .fetch_all(pool)
    .await
}

pub async fn find_security(pool: &PgPool, symbol: &str) -> sqlx::Result<Option<Security>> {
    sqlx::query_as(
        "SELECT s.symbol, s.isin, MIN(q.as_of) AS first_seen, MAX(q.as_of) AS last_seen,
                COUNT(q.id) AS data_points
         FROM securities s LEFT JOIN quotes q ON q.security_id = s.id
         WHERE s.symbol = UPPER($1)
         GROUP BY s.id",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HistoryPoint {
    pub as_of: DateTime<Utc>,
    pub open: f64,
    pub close: f64,
    pub change: f64,
    pub change_pct: f64,
    pub volume: f64,
    pub turnover: f64,
}

/// Price history for one symbol, oldest first. Returns the newest `limit` points in the range.
pub async fn history(
    pool: &PgPool,
    symbol: &str,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: i64,
) -> sqlx::Result<Vec<HistoryPoint>> {
    let rows: Vec<QuoteRow> = sqlx::query_as(
        "SELECT * FROM (
             SELECT s.symbol, s.isin, q.as_of, q.open_price, q.close_price, q.volume, q.turnover
             FROM quotes q JOIN securities s ON s.id = q.security_id
             WHERE s.symbol = UPPER($1)
               AND ($2::timestamptz IS NULL OR q.as_of >= $2)
               AND ($3::timestamptz IS NULL OR q.as_of <= $3)
             ORDER BY q.as_of DESC
             LIMIT $4
         ) recent ORDER BY as_of ASC",
    )
    .bind(symbol)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let as_of = row.as_of;
            let q = row.into_quote();
            HistoryPoint {
                as_of,
                open: q.open,
                close: q.close,
                change: q.change,
                change_pct: q.change_pct,
                volume: q.volume,
                turnover: q.turnover,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    //! These tests need Postgres. `sqlx::test` creates a fresh database for each test from
    //! `DATABASE_URL` and applies the migrations.
    use super::*;
    use chrono::TimeZone;

    fn snapshot(day: u32, quotes: Vec<Quote>) -> MarketSnapshot {
        MarketSnapshot {
            as_of: Utc.with_ymd_and_hms(2026, 11, day, 13, 0, 0).unwrap(),
            status: MarketStatus::Closed,
            quotes,
        }
    }

    fn quote(symbol: &str, open: f64, close: f64) -> Quote {
        Quote::new(
            symbol,
            Some(format!("MW{symbol}")),
            open,
            close,
            100.0,
            1000.0,
        )
    }

    #[sqlx::test]
    async fn saving_twice_does_not_duplicate(pool: PgPool) {
        let snap = snapshot(
            20,
            vec![quote("AIRTEL", 100.0, 101.0), quote("TNM", 20.0, 19.5)],
        );
        assert_eq!(save_snapshot(&pool, &snap).await.unwrap(), 2);
        assert_eq!(save_snapshot(&pool, &snap).await.unwrap(), 0);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quotes")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[sqlx::test]
    async fn latest_snapshot_uses_newest_quote_per_security(pool: PgPool) {
        assert_eq!(latest_snapshot(&pool).await.unwrap(), None);

        save_snapshot(
            &pool,
            &snapshot(
                19,
                vec![quote("AIRTEL", 99.0, 100.0), quote("TNM", 20.0, 20.0)],
            ),
        )
        .await
        .unwrap();
        save_snapshot(&pool, &snapshot(20, vec![quote("AIRTEL", 100.0, 101.0)]))
            .await
            .unwrap();

        let latest = latest_snapshot(&pool).await.unwrap().unwrap();
        assert_eq!(latest.as_of, snapshot(20, vec![]).as_of);
        let airtel = latest.quotes.iter().find(|q| q.symbol == "AIRTEL").unwrap();
        assert_eq!(airtel.close, 101.0);
        assert!(
            latest.quotes.iter().any(|q| q.symbol == "TNM"),
            "securities that didn't trade keep their last quote"
        );
    }

    #[sqlx::test]
    async fn history_is_filtered_limited_and_ascending(pool: PgPool) {
        for day in 16..=20 {
            save_snapshot(
                &pool,
                &snapshot(day, vec![quote("NBM", 100.0, 100.0 + day as f64)]),
            )
            .await
            .unwrap();
        }

        let all = history(&pool, "nbm", None, None, 100).await.unwrap();
        assert_eq!(
            all.iter().map(|p| p.close).collect::<Vec<_>>(),
            [116.0, 117.0, 118.0, 119.0, 120.0]
        );

        let last_two = history(&pool, "NBM", None, None, 2).await.unwrap();
        assert_eq!(
            last_two.iter().map(|p| p.close).collect::<Vec<_>>(),
            [119.0, 120.0]
        );

        let from = snapshot(18, vec![]).as_of;
        let ranged = history(&pool, "NBM", Some(from), None, 100).await.unwrap();
        assert_eq!(ranged.len(), 3);

        assert!(
            history(&pool, "NOPE", None, None, 100)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[sqlx::test]
    async fn lists_and_finds_securities(pool: PgPool) {
        save_snapshot(
            &pool,
            &snapshot(19, vec![quote("TNM", 1.0, 1.0), quote("AIRTEL", 1.0, 1.0)]),
        )
        .await
        .unwrap();
        save_snapshot(&pool, &snapshot(20, vec![quote("TNM", 1.0, 1.0)]))
            .await
            .unwrap();

        let list = list_securities(&pool).await.unwrap();
        assert_eq!(
            list.iter().map(|s| s.symbol.as_str()).collect::<Vec<_>>(),
            ["AIRTEL", "TNM"]
        );
        assert_eq!(list[1].data_points, 2);

        let tnm = find_security(&pool, "tnm").await.unwrap().unwrap();
        assert_eq!(tnm.isin.as_deref(), Some("MWTNM"));
        assert!(find_security(&pool, "NOPE").await.unwrap().is_none());
    }
}
