// api/routes.rs
use actix_web::web;
use std::sync::Arc;
use std::env;
use tokio::sync::Mutex;
// use chrono::{DateTime, Utc};

use crate::api::handlers::{get_all_stocks, get_current_stocks, health_check, StockCache};
use crate::api::ws::stock_ws;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    let cache = web::Data::new(StockCache {
        data: Arc::new(Mutex::new(None)),
        last_updated: Arc::new(Mutex::new(None)),
    });

    cfg.app_data(cache.clone())
       .app_data(web::Data::new(env::var("STOCK_API_URL").unwrap()))
       .route("/stocks", web::get().to(get_all_stocks))
       .route("/stocks/current", web::get().to(get_current_stocks))
       .route("/stocks/ws", web::get().to(stock_ws))
       .route("/health", web::get().to(health_check));
}