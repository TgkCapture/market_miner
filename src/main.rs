// main.rs
mod db;
mod scraper;
mod models;
mod utils;
mod api;

use db::{create_database_if_not_exists, connect_db};
use scraper::job::start_scraping;
use utils::logging::{log_error, log_info};
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
// use chrono::Utc;
use actix_web::{App, HttpServer, web::Data};
use api::routes::configure_routes;
use api::handlers::StockCache;

#[tokio::main]
async fn main() {
    dotenv().ok();
    log_info("Starting Market Miner...");

    // Retrieve API host and port from .env
    let api_host = env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let api_port = env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
    let api_address = format!("{}:{}", api_host, api_port);

    // Initialize the database
    if let Err(e) = create_database_if_not_exists().await {
        log_error(&format!("Failed to create database: {}", e));
        return;
    }

    let client = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            log_error(&format!("Failed to connect to database: {}", e));
            return;
        }
    };

    // Create the shared cache
    let cache = Data::new(StockCache {
        data: Arc::new(Mutex::new(None)),
        last_updated: Arc::new(Mutex::new(None)),
    });

    let cache_for_server = cache.clone(); 
    let cache_for_scraper = cache.clone(); 

    // Start the Actix-web server and the scraping loop concurrently
    let server = HttpServer::new(move || {
        App::new()
            .app_data(cache_for_server.clone())
            .configure(configure_routes)
    })
    .bind(&api_address)
    .unwrap_or_else(|e| {
        log_error(&format!("Failed to start API server on {}: {}", api_address, e));
        std::process::exit(1);
    });

    log_info(&format!("API server running on http://{}", api_address));
    println!("API server running on http://{}", api_address);

    tokio::select! {
        server_result = server.run() => {
            if let Err(e) = server_result {
                log_error(&format!("API server error: {}", e));
            }
        }
        _ = async {
            // Start the scraping loop
            let url = env::var("STOCK_API_URL").expect("STOCK_API_URL must be set");
            let fetch_interval = env::var("FETCH_INTERVAL")
                .unwrap_or_else(|_| "300".to_string())
                .parse::<u64>()
                .expect("FETCH_INTERVAL must be a valid number");

            start_scraping(url, fetch_interval, client, cache_for_scraper.clone()).await;
        } => {}
    }
}