use axum::{Json, response::IntoResponse};
use serde_json::json;
use crate::db::queries;
use crate::db::connection::get_db_connection;
use crate::models::stock::Stock;
use std::sync::Arc;

pub async fn get_all_stocks() -> impl IntoResponse {
    let conn = get_db_connection().await;
    match queries::get_all_stocks(&conn).await {
        Ok(stocks) => Json(stocks),
        Err(err) => Json(json!({"error": format!("Database error: {}", err)})),
    }
}

pub async fn health_check() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}
