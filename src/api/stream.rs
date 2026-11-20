//! Live updates over Server-Sent Events. Each event is a `market` event whose data is the
//! same JSON as `GET /api/market/latest`. The current snapshot is sent as soon as a client connects.

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

use super::market::MarketSummary;
use crate::market::MarketSnapshot;
use crate::state::AppState;

pub async fn stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Subscribe before reading the current snapshot so no update is missed in between.
    let updates = BroadcastStream::new(state.subscribe())
        // A lagging client skips missed snapshots; the next one has the full board anyway.
        .filter_map(|msg| msg.ok());
    let initial = tokio_stream::iter(state.snapshot().await);

    let events = initial.chain(updates).map(|snap| Ok(to_event(&snap)));
    Sse::new(events).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

fn to_event(snapshot: &MarketSnapshot) -> Event {
    Event::default()
        .event("market")
        .json_data(MarketSummary::from_snapshot(snapshot))
        .expect("market summary serializes to JSON")
}
