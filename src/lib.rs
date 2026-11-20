//! Market Miner: scrapes Malawi Stock Exchange prices, stores their history in Postgres and
//! serves them through a JSON API, a live Server-Sent Events stream and a web dashboard.

pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod market;
pub mod scraper;
pub mod state;
