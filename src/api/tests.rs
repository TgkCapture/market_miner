//! Router tests. The first group runs without a database; the `sqlx::test` ones need Postgres.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::{TimeZone, Utc};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

use super::router;
use crate::db;
use crate::market::{MarketSnapshot, MarketStatus, Quote};
use crate::state::AppState;

fn snapshot() -> MarketSnapshot {
    MarketSnapshot {
        as_of: Utc.with_ymd_and_hms(2026, 11, 20, 13, 0, 0).unwrap(),
        status: MarketStatus::Closed,
        quotes: vec![
            Quote::new(
                "CHL",
                Some("MWCHL0010175".into()),
                193.04,
                194.91,
                15_076.0,
                2_938_484.0,
            ),
            Quote::new(
                "TNM",
                Some("MWTNM0010126".into()),
                20.62,
                20.57,
                254_837.0,
                5_241_039.0,
            ),
            Quote::new("PCL", None, 3300.0, 3300.0, 0.0, 0.0),
        ],
    }
}

fn offline_state() -> AppState {
    AppState::new(PgPool::connect_lazy("postgres://unused@localhost/unused").unwrap())
}

async fn get(state: &AppState, uri: &str) -> (StatusCode, Value) {
    let res = router(state.clone(), None)
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn health_reports_ok_before_any_data() {
    let (status, body) = get(&offline_state(), "/api/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert!(body["as_of"].is_null());
}

#[tokio::test]
async fn latest_is_unavailable_until_first_scrape() {
    let (status, body) = get(&offline_state(), "/api/market/latest").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body["error"].as_str().unwrap().contains("no market data"));
}

#[tokio::test]
async fn latest_returns_snapshot_with_totals() {
    let state = offline_state();
    state.publish(snapshot()).await;
    let (status, body) = get(&state, "/api/market/latest").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "closed");
    assert_eq!(body["as_of"], "2026-11-20T13:00:00Z");
    assert_eq!(body["quotes"].as_array().unwrap().len(), 3);
    assert_eq!(body["quotes"][0]["change"], 1.87);
    assert_eq!(body["advancers"], 1);
    assert_eq!(body["decliners"], 1);
    assert_eq!(body["unchanged"], 1);
    assert_eq!(body["total_turnover"], 8_179_523.0);
}

#[tokio::test]
async fn movers_respects_limit() {
    let state = offline_state();
    state.publish(snapshot()).await;
    let (status, body) = get(&state, "/api/market/movers?limit=1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["gainers"][0]["symbol"], "CHL");
    assert_eq!(body["losers"][0]["symbol"], "TNM");
    assert_eq!(body["most_traded"][0]["symbol"], "TNM");
    assert_eq!(body["most_traded"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn unknown_api_path_is_json_404() {
    let (status, body) = get(&offline_state(), "/api/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "no such endpoint");
}

#[tokio::test]
async fn stream_sends_current_snapshot_first() {
    let state = offline_state();
    state.publish(snapshot()).await;
    let res = router(state, None)
        .oneshot(Request::get("/api/stream").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()[header::CONTENT_TYPE], "text/event-stream");

    let mut body = res.into_body();
    let frame = body.frame().await.unwrap().unwrap().into_data().unwrap();
    let text = String::from_utf8(frame.to_vec()).unwrap();
    assert!(text.starts_with("event: market\ndata: {"), "{text}");
    assert!(text.contains("\"symbol\":\"CHL\""));
}

#[tokio::test]
async fn serves_dashboard_with_spa_fallback() {
    let dir = std::env::temp_dir().join(format!("mm-static-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), "<h1>dashboard</h1>").unwrap();

    for uri in ["/", "/stock/TNM"] {
        let res = router(offline_state(), Some(&dir))
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK, "{uri}");
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&bytes[..], b"<h1>dashboard</h1>");
    }
    std::fs::remove_dir_all(dir).ok();
}

#[sqlx::test]
async fn securities_and_history(pool: PgPool) {
    let state = AppState::new(pool.clone());
    db::save_snapshot(&pool, &snapshot()).await.unwrap();
    state.publish(snapshot()).await;

    let (status, body) = get(&state, "/api/securities").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 3);

    let (status, body) = get(&state, "/api/securities/chl").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["symbol"], "CHL");
    assert_eq!(body["isin"], "MWCHL0010175");
    assert_eq!(body["quote"]["close"], 194.91);

    let (status, body) = get(&state, "/api/securities/TNM/history").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["symbol"], "TNM");
    assert_eq!(body["points"][0]["close"], 20.57);

    let (status, _) = get(&state, "/api/securities/NOPE/history").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = get(
        &state,
        "/api/securities/TNM/history?from=2026-11-21T00:00:00Z&to=2026-11-20T00:00:00Z",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
