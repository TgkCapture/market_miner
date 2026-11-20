//! HTTP routes. JSON API under `/api`, the dashboard everywhere else.

mod market;
mod securities;
mod stream;

use std::path::Path;

use axum::Router;
use axum::routing::get;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::error::AppError;
use crate::state::AppState;

pub fn router(state: AppState, static_dir: Option<&Path>) -> Router {
    let api = Router::new()
        .route("/health", get(market::health))
        .route("/market/latest", get(market::latest))
        .route("/market/movers", get(market::movers))
        .route("/securities", get(securities::list))
        .route("/securities/{symbol}", get(securities::detail))
        .route("/securities/{symbol}/history", get(securities::history))
        .route("/stream", get(stream::stream))
        .fallback(|| async { AppError::NotFound("no such endpoint".into()) })
        .with_state(state);

    let mut app = Router::new().nest("/api", api);

    if let Some(dir) = static_dir.filter(|d| d.join("index.html").is_file()) {
        // Unknown paths fall back to index.html so client-side routes like /stock/TNM work on reload.
        let spa = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
        app = app.fallback_service(spa);
    } else {
        tracing::warn!("dashboard not built; serving the API only");
    }

    // The default compression predicate skips text/event-stream, so SSE isn't buffered.
    app.layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests;
