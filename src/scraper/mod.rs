//! Fetching and parsing the MSE mainboard page, plus the background job that does it on a timer.

pub mod job;
pub mod parse;

use std::time::Duration;

use crate::market::MarketSnapshot;

#[derive(Debug, thiserror::Error)]
pub enum ScrapeError {
    #[error("request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Parse(#[from] parse::ParseError),
}

/// HTTP client for the exchange website.
#[derive(Clone)]
pub struct MseClient {
    http: reqwest::Client,
    url: String,
}

impl MseClient {
    pub fn new(url: impl Into<String>) -> reqwest::Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("market-miner/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            http,
            url: url.into(),
        })
    }

    pub async fn fetch_snapshot(&self) -> Result<MarketSnapshot, ScrapeError> {
        let html = self
            .http
            .get(&self.url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(parse::parse_mainboard(&html)?)
    }
}
