use reqwest::Error;
use crate::models::stock::Stock;
use serde_json::Value;

/// Fetches stock data from the given API URL
pub async fn fetch_stock_data(url: &str) -> Result<Vec<Stock>, Error> {
    let response = reqwest::get(url).await?.json::<Value>().await?;

    let stocks = response["stocks"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|stock| Stock {
            id: 0, 
            symbol: stock["symbol"].as_str().unwrap_or("").to_string(),
            open_price: stock["open_price"].as_f64().unwrap_or(0.0),
            close_price: stock["close_price"].as_f64().unwrap_or(0.0),
            percent_change: stock["percent_change"].as_f64().unwrap_or(0.0),
            volume: stock["volume"].as_f64().unwrap_or(0.0),
            turnover: stock["turnover"].as_f64().unwrap_or(0.0),
            timestamp: stock["timestamp"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Ok(stocks)
}
