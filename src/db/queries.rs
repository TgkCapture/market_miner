// db/queries.rs
use tokio_postgres::Client;
use crate::models::stock::Stock;
use tokio_postgres::Error;
use chrono::{DateTime, Utc};

/// Inserts stock data while preserving historical records
pub async fn insert_stock_data(
    client: &Client, 
    stocks: Vec<Stock>
) -> Result<(), Error> {
    for stock in stocks {
        let result = client.execute(
            "INSERT INTO stocks (symbol, open_price, close_price, percent_change, volume, turnover, timestamp) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &stock.symbol,
                &stock.open_price,
                &stock.close_price,
                &stock.percent_change,
                &stock.volume,
                &stock.turnover,
                &stock.timestamp, 
            ],
        ).await;

        if let Err(e) = result {
            eprintln!("Failed to insert stock data for '{}': {}", stock.symbol, e);
            return Err(e);
        }
    }
    Ok(())
}

/// Retrieves all stocks from the database
pub async fn get_all_stocks(client: &Client) -> Result<Vec<Stock>, Error> {
    let rows = client.query("SELECT id, symbol, open_price, close_price, percent_change, volume, turnover, timestamp FROM stocks", &[]).await?;
    
    let stocks = rows.iter().map(|row| Stock {
        id: row.get(0),
        symbol: row.get(1),
        open_price: row.get(2),
        close_price: row.get(3),
        percent_change: row.get(4),
        volume: row.get(5),
        turnover: row.get(6),
        timestamp: row.get::<_, DateTime<Utc>>(7), 
    }).collect();

    Ok(stocks)
}