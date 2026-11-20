use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};
use crate::market::{MarketSnapshot, Movers};
use crate::state::{AppState, ScrapeStatus};

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
    version: &'static str,
    as_of: Option<DateTime<Utc>>,
    scrape: ScrapeStatus,
}

pub async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        as_of: state.snapshot().await.map(|s| s.as_of),
        scrape: state.scrape_status().await,
    })
}

/// The latest snapshot plus board-wide totals.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct MarketSummary {
    #[serde(flatten)]
    pub snapshot: MarketSnapshot,
    pub total_turnover: f64,
    pub total_volume: f64,
    pub advancers: usize,
    pub decliners: usize,
    pub unchanged: usize,
}

impl MarketSummary {
    pub fn from_snapshot(snapshot: &MarketSnapshot) -> Self {
        let count = |f: fn(f64) -> bool| snapshot.quotes.iter().filter(|q| f(q.change)).count();
        Self {
            total_turnover: snapshot.total_turnover(),
            total_volume: snapshot.total_volume(),
            advancers: count(|c| c > 0.0),
            decliners: count(|c| c < 0.0),
            unchanged: count(|c| c == 0.0),
            snapshot: snapshot.clone(),
        }
    }
}

pub(crate) async fn current(state: &AppState) -> ApiResult<Arc<MarketSnapshot>> {
    state.snapshot().await.ok_or_else(|| {
        AppError::Unavailable("no market data yet; the first scrape hasn't finished".into())
    })
}

pub async fn latest(State(state): State<AppState>) -> ApiResult<Json<MarketSummary>> {
    Ok(Json(MarketSummary::from_snapshot(&*current(&state).await?)))
}

#[derive(Deserialize)]
pub struct MoversQuery {
    limit: Option<usize>,
}

pub async fn movers(
    State(state): State<AppState>,
    Query(q): Query<MoversQuery>,
) -> ApiResult<Json<Movers>> {
    let limit = q.limit.unwrap_or(5).clamp(1, 50);
    Ok(Json(current(&state).await?.movers(limit)))
}
