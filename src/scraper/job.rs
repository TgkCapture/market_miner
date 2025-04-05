// scraper/job.rs
use tokio::time::{self, Duration};
use crate::scraper::scraper::fetch_stock_data;
use crate::db::{insert_stock_data, Client};
use crate::utils::logging::{log_info, log_error};

/// Starts the scraping job with the given URL, interval, and database client
pub async fn start_scraping(url: String, interval: u64, client: Client, cache: web::Data<StockCache>) {
    let fetch_interval = Duration::from_secs(interval);

    loop {
        log_info("Fetching stock data...");

        match fetch_stock_data(&url).await {
            Ok(stocks) => {
                *cache.data.lock().await = Some(stocks.clone());
                *cache.last_updated.lock().await = Some(Utc::now());
                
                let num_stocks = stocks.len();
                log_info(&format!("Fetched {} stocks", num_stocks));

                if num_stocks == 0 {
                    log_error("No stocks were fetched from the API.");
                }

                // Insert the fetched stocks into the database
                if let Err(e) = insert_stock_data(&client, stocks).await {
                    log_error(&format!("Failed to insert stock data: {}", e));
                }
            }
            Err(e) => log_error(&format!("Error fetching stock data: {}", e)),
        }

        log_info(&format!("Waiting for {} seconds before next fetch...", fetch_interval.as_secs()));
        time::sleep(fetch_interval).await;
    }
}