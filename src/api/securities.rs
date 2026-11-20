use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::db::{self, HistoryPoint, Security};
use crate::error::{ApiResult, AppError};
use crate::market::Quote;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<Security>>> {
    Ok(Json(db::list_securities(&state.pool).await?))
}

#[derive(Serialize)]
pub struct SecurityDetail {
    #[serde(flatten)]
    security: Security,
    /// The security's quote in the latest snapshot, if it has one.
    quote: Option<Quote>,
}

pub async fn detail(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<SecurityDetail>> {
    let security = db::find_security(&state.pool, &symbol)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("unknown symbol {symbol:?}")))?;
    let quote = state.snapshot().await.and_then(|s| {
        s.quotes
            .iter()
            .find(|q| q.symbol == security.symbol)
            .cloned()
    });
    Ok(Json(SecurityDetail { security, quote }))
}

pub const DEFAULT_HISTORY_LIMIT: i64 = 500;
pub const MAX_HISTORY_LIMIT: i64 = 5000;

#[derive(Deserialize)]
pub struct HistoryQuery {
    /// RFC 3339 timestamps, e.g. `2026-11-01T00:00:00Z`.
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: Option<i64>,
}

#[derive(Serialize)]
pub struct History {
    symbol: String,
    points: Vec<HistoryPoint>,
}

pub async fn history(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<HistoryQuery>,
) -> ApiResult<Json<History>> {
    if let (Some(from), Some(to)) = (q.from, q.to) {
        if from > to {
            return Err(AppError::BadRequest("`from` must be before `to`".into()));
        }
    }
    let limit = q
        .limit
        .unwrap_or(DEFAULT_HISTORY_LIMIT)
        .clamp(1, MAX_HISTORY_LIMIT);
    let security = db::find_security(&state.pool, &symbol)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("unknown symbol {symbol:?}")))?;
    let points = db::history(&state.pool, &security.symbol, q.from, q.to, limit).await?;
    Ok(Json(History {
        symbol: security.symbol,
        points,
    }))
}
