use axum::Json;
use serde_json::json;

/// Creates a JSON response for success cases
pub fn success_response<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({
        "status": "success",
        "data": data
    }))
}

/// Creates a JSON response for error cases
pub fn error_response(message: &str) -> Json<serde_json::Value> {
    Json(json!({
        "status": "error",
        "message": message
    }))
}
