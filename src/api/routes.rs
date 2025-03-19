// api/routes.rs
use actix_web::web;
use crate::api::handlers::{get_all_stocks, health_check};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/stocks", web::get().to(get_all_stocks))
       .route("/health", web::get().to(health_check));
}