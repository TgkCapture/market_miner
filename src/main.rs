mod db;
mod scraper;
mod models;
mod utils;

use db::{create_database_if_not_exists, connect_db, insert_stock_data};
use scraper::fetch_stock_data;
use utils::{log_error, log_info};
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    log_info("Starting Market Miner...");

    create_database_if_not_exists().await.expect("Failed to create database");
    
    let client = connect_db().await.expect("Failed to connect to database");

    let url = env::var("STOCK_API_URL").expect("STOCK_API_URL must be set");
    match fetch_stock_data(&url).await {
        Ok(stocks) => {
            log_info(&format!("Fetched {} stocks", stocks.len()));
            insert_stock_data(&client, stocks).await.expect("Failed to insert stock data");
        }
        Err(e) => log_error(&format!("Error fetching stock data: {}", e)),
    }
}