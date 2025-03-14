use axum::{routing::get, Router};
use crate::api::handlers::{get_all_stocks, health_check};

pub fn create_routes() -> Router {
    Router::new()
        .route("/stocks", get(get_all_stocks))
        .route("/health", get(health_check))
}
