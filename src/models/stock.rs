// models/stock.rs
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stock {
    pub id: i32, 
    pub symbol: String,
    pub open_price: f64,
    pub close_price: f64,
    pub percent_change: f64,
    pub volume: f64,
    pub turnover: f64,
    pub timestamp: DateTime<Utc>, 
}