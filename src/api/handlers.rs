// api/handlers.rs
use actix_web::{HttpResponse};
use crate::db::queries;
use crate::db::connection::connect_db;
use crate::api::response::{success_response, error_response};

// Handler to get all stocks
pub async fn get_all_stocks() -> HttpResponse {
    let conn = match connect_db().await {
        Ok(client) => client,
        Err(err) => {
            return error_response(&format!("Database connection failed: {}", err));
        }
    };

    match queries::get_all_stocks(&conn).await {
        Ok(stocks) => success_response(stocks),
        Err(err) => error_response(&format!("Database error: {}", err)),
    }
}

// Health check handler
pub async fn health_check() -> HttpResponse {
    success_response("ok")
}