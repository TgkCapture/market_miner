// api/handlers.rs
use actix_web::{HttpResponse, web};
// use actix_web::web::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc, Duration};  

use crate::models::stock::Stock; 
use crate::db::queries;
use crate::db::insert_stock_data;
use crate::db::connection::connect_db;
use crate::api::response::{success_response, error_response};
use crate::scraper::scraper::fetch_stock_data;

// Shared in-memory cache
pub struct StockCache {
    pub data: Arc<Mutex<Option<Vec<Stock>>>>,
    pub last_updated: Arc<Mutex<Option<DateTime<Utc>>>>,
}

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

// Real-time endpoint
pub async fn get_current_stocks(
    cache: web::Data<StockCache>,
    scraper_url: web::Data<String>,
) -> HttpResponse {
    let now = Utc::now();
    let last_updated = *cache.last_updated.lock().await;
    let is_fresh = last_updated.map_or(false, |t| (now - t) < Duration::seconds(5));

    if is_fresh {
        if let Some(data) = &*cache.data.lock().await {
            return success_response(data.clone());
        }
    }

    // Direct scrape if cache stale
    match fetch_stock_data(&scraper_url).await {
        Ok(stocks) => {
            let stocks_clone = stocks.clone();
            *cache.data.lock().await = Some(stocks.clone());
            *cache.last_updated.lock().await = Some(Utc::now());
            
            tokio::spawn(async move {
                if let Ok(client) = connect_db().await {
                    let _ = insert_stock_data(&client, &stocks_clone).await;  
                }
            });

            success_response(stocks)  
        }
        Err(e) => error_response(&format!("Scraping error: {}", e)),
    }
}
