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
// use tokio::time::{self, Duration};
use actix_web::{App, HttpServer};
use api::routes::configure_routes;

#[tokio::main]
async fn main() {
    dotenv().ok();
    log_info("Starting Market Miner...");

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

    // Start the Actix-web server and the scraping loop concurrently
    let server = HttpServer::new(|| {
        App::new()
            .configure(configure_routes)
    })
    .bind("127.0.0.1:3000") 
    .unwrap_or_else(|e| {
        log_error(&format!("Failed to start API server: {}", e));
        std::process::exit(1);
    });

    log_info("API server running on http://127.0.0.1:3000");

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

            start_scraping(url, fetch_interval, client).await;
        } => {}
    }
}