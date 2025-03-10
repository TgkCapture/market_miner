mod config;
mod models;
mod scraper;
mod utils;

use config::get_env_var;
use scraper::fetch_stock_data;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting stock scraper...");

    let url = get_env_var("STOCK_API_URL");
    log::info!("Fetching data from: {}", url);

    match fetch_stock_data(&url).await {
        Ok(stocks) => {
            log::info!("Fetched {} stocks", stocks.len());
            for stock in stocks {
                println!(
                    "Symbol: {}, Open: {}, Close: {}, % Change: {}, Volume: {}, Turnover: {}",
                    stock.symbol,
                    stock.open_price,
                    stock.close_price,
                    stock.percent_change,
                    stock.volume,
                    stock.turnover
                );
            }
        }
        Err(e) => log::error!("Error fetching stock data: {}", e),
    }
}