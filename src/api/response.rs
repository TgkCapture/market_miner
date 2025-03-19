// api/response.rs
use actix_web::HttpResponse;
use serde_json::json;

/// Creates a JSON response for success cases
pub fn success_response<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "success",
        "data": data
    }))
}

/// Creates a JSON response for error cases
pub fn error_response(message: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({
        "status": "error",
        "message": message
    }))
}