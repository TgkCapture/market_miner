mod config;
mod models;
mod scraper;
mod utils;

use config::get_env_var;
use scraper::fetch_stock_data;
use utils::{log_info, save_to_json};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting stock scraper...");

    let url = get_env_var("STOCK_API_URL");
    log::info!("Fetching data from: {}", url);

    match fetch_stock_data(&url).await {
        Ok(stocks) => {
            log_info(&format!("Fetched {} stocks", stocks.len()));
            save_to_json(&stocks, "data/stocks.json");
        }
        Err(e) => log::error!("Error fetching stock data: {}", e),
    }
}