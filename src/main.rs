use anyhow::Context;
use tracing_subscriber::EnvFilter;

use market_miner::config::Config;
use market_miner::scraper::{MseClient, job};
use market_miner::state::AppState;
use market_miner::{api, db};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("market_miner=info,tower_http=info")),
        )
        .init();

    let config = Config::from_env()?;

    let pool = db::connect(&config.database_url)
        .await
        .context("connecting to Postgres")?;
    db::migrate(&pool)
        .await
        .context("running database migrations")?;

    let state = AppState::new(pool);
    if let Some(snapshot) = db::latest_snapshot(&state.pool).await? {
        tracing::info!(as_of = %snapshot.as_of, "loaded latest snapshot from the database");
        state.publish(snapshot).await;
    }

    if config.scraper_enabled {
        let client = MseClient::new(&config.mse_url)?;
        tracing::info!(url = %config.mse_url, every = ?config.scrape_interval, "starting scraper");
        tokio::spawn(job::run(state.clone(), client, config.scrape_interval));
    } else {
        tracing::warn!("scraper disabled (SCRAPER_ENABLED=false)");
    }

    let app = api::router(state, Some(&config.static_dir));
    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .with_context(|| format!("binding {}", config.bind_addr))?;
    tracing::info!("listening on http://{}", config.bind_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutting down");
}
