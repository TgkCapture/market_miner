mod config;
mod models;
mod scraper;
mod utils;

use config::get_env_var;
use scraper::fetch_stock_data;
use utils::{log_error, log_info, log_warning, save_to_json};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log_info("Starting Market Miner...");

    let url = get_env_var("STOCK_API_URL");
    log_info(&format!("Fetching data from: {}", url));

    match fetch_stock_data(&url).await {
        Ok(stocks) => {
            log_info(&format!("Fetched {} stocks", stocks.len()));
            save_to_json(&stocks, "data/stocks.json");
        }
        Err(e) => {
            log_error(&format!("Error fetching stock data: {}", e));
            log_warning("Check your internet connection or the website URL.");
        }
    }
}