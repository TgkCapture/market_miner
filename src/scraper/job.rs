//! Background task: scrape on a timer, save new data and publish it to live subscribers.

use std::time::Duration;

use tokio::time::{MissedTickBehavior, interval};

use crate::db;
use crate::scraper::MseClient;
use crate::state::AppState;

pub async fn run(state: AppState, client: MseClient, every: Duration) {
    let mut ticker = interval(every);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        ticker.tick().await;
        scrape_once(&state, &client).await;
    }
}

pub async fn scrape_once(state: &AppState, client: &MseClient) {
    let result = async {
        let snapshot = client.fetch_snapshot().await.map_err(|e| e.to_string())?;
        let inserted = db::save_snapshot(&state.pool, &snapshot)
            .await
            .map_err(|e| format!("saving snapshot: {e}"))?;
        tracing::info!(
            as_of = %snapshot.as_of,
            securities = snapshot.quotes.len(),
            new_quotes = inserted,
            "scraped mainboard"
        );
        state.publish(snapshot).await;
        Ok::<(), String>(())
    }
    .await;

    if let Err(e) = &result {
        tracing::error!(error = %e, "scrape failed");
    }
    state.record_scrape(result).await;
}
