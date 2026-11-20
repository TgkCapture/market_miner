//! State shared between the scrape job and the HTTP handlers.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use tokio::sync::{RwLock, broadcast};

use crate::market::MarketSnapshot;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ScrapeStatus {
    pub last_attempt: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    inner: Arc<Inner>,
}

struct Inner {
    snapshot: RwLock<Option<Arc<MarketSnapshot>>>,
    scrape: RwLock<ScrapeStatus>,
    updates: broadcast::Sender<Arc<MarketSnapshot>>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let (updates, _) = broadcast::channel(16);
        Self {
            pool,
            inner: Arc::new(Inner {
                snapshot: RwLock::new(None),
                scrape: RwLock::new(ScrapeStatus::default()),
                updates,
            }),
        }
    }

    pub async fn snapshot(&self) -> Option<Arc<MarketSnapshot>> {
        self.inner.snapshot.read().await.clone()
    }

    /// Stores the snapshot and notifies live subscribers if the data changed.
    /// Returns true when it changed.
    pub async fn publish(&self, snapshot: MarketSnapshot) -> bool {
        let mut current = self.inner.snapshot.write().await;
        if current.as_deref() == Some(&snapshot) {
            return false;
        }
        let snapshot = Arc::new(snapshot);
        *current = Some(snapshot.clone());
        // An error only means nobody is listening right now.
        let _ = self.inner.updates.send(snapshot);
        true
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<MarketSnapshot>> {
        self.inner.updates.subscribe()
    }

    pub async fn scrape_status(&self) -> ScrapeStatus {
        self.inner.scrape.read().await.clone()
    }

    pub async fn record_scrape(&self, result: Result<(), String>) {
        let mut status = self.inner.scrape.write().await;
        let now = Utc::now();
        status.last_attempt = Some(now);
        match result {
            Ok(()) => {
                status.last_success = Some(now);
                status.last_error = None;
            }
            Err(e) => status.last_error = Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::{MarketStatus, Quote};

    fn lazy_pool() -> PgPool {
        PgPool::connect_lazy("postgres://unused@localhost/unused").unwrap()
    }

    #[tokio::test]
    async fn publish_only_notifies_on_change() {
        let state = AppState::new(lazy_pool());
        let mut rx = state.subscribe();
        let snap = MarketSnapshot {
            as_of: Utc::now(),
            status: MarketStatus::Open,
            quotes: vec![Quote::new("A", None, 1.0, 2.0, 1.0, 1.0)],
        };

        assert!(state.publish(snap.clone()).await);
        assert_eq!(*rx.recv().await.unwrap(), snap);

        assert!(!state.publish(snap.clone()).await);
        assert!(rx.try_recv().is_err());
        assert_eq!(state.snapshot().await.as_deref(), Some(&snap));
    }

    #[tokio::test]
    async fn records_scrape_results() {
        let state = AppState::new(lazy_pool());
        state.record_scrape(Err("boom".into())).await;
        let s = state.scrape_status().await;
        assert_eq!(s.last_error.as_deref(), Some("boom"));
        assert!(s.last_success.is_none());

        state.record_scrape(Ok(())).await;
        let s = state.scrape_status().await;
        assert!(s.last_error.is_none());
        assert!(s.last_success.is_some());
    }
}
