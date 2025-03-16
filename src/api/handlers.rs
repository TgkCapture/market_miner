use axum::{Json, response::IntoResponse};
use serde_json::json;
use crate::db::queries;
use crate::db::connection::connect_db;

pub async fn get_all_stocks() -> impl IntoResponse {
    let conn = match connect_db().await {
        Ok(client) => client,
        Err(err) => return Json(json!({"error": format!("Database connection failed: {}", err)})),
    };

    match queries::get_all_stocks(&conn).await {
        Ok(stocks) => Json(json!({ "status": "success", "data": stocks })),  
        Err(err) => Json(json!({ "status": "error", "message": format!("Database error: {}", err) })),
    }
}

pub async fn health_check() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}
