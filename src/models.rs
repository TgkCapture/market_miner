use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Serialize, Debug)]
pub struct Stock {
    pub symbol: String,
    pub open_price: f64,
    pub close_price: f64,
    pub percent_change: f64,
    pub volume: f64,
    pub turnover: f64,
    pub timestamp: DateTime<Utc>,
}