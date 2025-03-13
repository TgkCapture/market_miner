mod db;
mod scraper;
mod models;
mod utils;

use db::{create_database_if_not_exists, connect_db, insert_stock_data};
use scraper::fetch_stock_data;
use utils::{log_error, log_info, log_warning};
use dotenvy::dotenv;
use std::env;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    dotenv().ok();
    log_info("Starting Market Miner...");

    // Create the database if it doesn't exist
    if let Err(e) = create_database_if_not_exists().await {
        log_error(&format!("Failed to create database: {}", e));
        return; 
    }

    // Connect to the database
    let client = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            log_error(&format!("Failed to connect to database: {}", e));
            return;
        }
    };

    // Define the interval for fetching stock data (e.g., every 5 minutes)
    let fetch_interval = Duration::from_secs(300); // 300 seconds = 5 minutes

    // Main loop to periodically fetch and insert stock data
    loop {
        log_info("Fetching stock data...");

        // Fetch stock data
        let url = env::var("STOCK_API_URL").expect("STOCK_API_URL must be set");
        match fetch_stock_data(&url).await {
            Ok(stocks) => {
                let num_stocks = stocks.len();
                log_info(&format!("Fetched {} stocks", num_stocks));

                // Log a warning if no stocks were fetched
                if num_stocks == 0 {
                    log_warning("No stocks were fetched from the API.");
                }

                // Insert stock data into the database
                if let Err(e) = insert_stock_data(&client, stocks).await {
                    log_error(&format!("Failed to insert stock data: {}", e));
                }
            }
            Err(e) => log_error(&format!("Error fetching stock data: {}", e)),
        }

        // Wait for the next fetch interval
        log_info(&format!("Waiting for {} seconds before next fetch...", fetch_interval.as_secs()));
        time::sleep(fetch_interval).await;
    }
}