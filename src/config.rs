//! Runtime configuration, read once from environment variables at startup.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

pub const DEFAULT_MSE_URL: &str = "https://mse.co.mw/market/mainboard";

#[derive(Debug, Clone)]
pub struct Config {
    /// Postgres connection string.
    pub database_url: String,
    /// Address the HTTP server binds to.
    pub bind_addr: SocketAddr,
    /// Page the scraper reads.
    pub mse_url: String,
    /// Time between scrapes.
    pub scrape_interval: Duration,
    /// Set to false to run the API without scraping (useful in development).
    pub scraper_enabled: bool,
    /// Directory holding the built dashboard. Served at `/` when it exists.
    pub static_dir: PathBuf,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ConfigError {
    #[error("{0} must be set")]
    Missing(&'static str),
    #[error("{key} has an invalid value {value:?}: {reason}")]
    Invalid {
        key: &'static str,
        value: String,
        reason: &'static str,
    },
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| std::env::var(key).ok().filter(|v| !v.trim().is_empty()))
    }

    /// Builds the config from any key/value source, so it can be tested without touching
    /// the process environment.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let database_url = get("DATABASE_URL").ok_or(ConfigError::Missing("DATABASE_URL"))?;

        let bind_addr = parse_or(
            &get,
            "BIND_ADDR",
            "0.0.0.0:8080",
            |v| v.parse().ok(),
            "expected host:port",
        )?;
        let interval_secs: u64 = parse_or(
            &get,
            "SCRAPE_INTERVAL_SECS",
            "300",
            |v| v.parse().ok().filter(|n| *n >= 10),
            "expected a whole number of seconds, at least 10",
        )?;
        let scraper_enabled = parse_or(
            &get,
            "SCRAPER_ENABLED",
            "true",
            parse_bool,
            "expected true or false",
        )?;

        Ok(Self {
            database_url,
            bind_addr,
            mse_url: get("MSE_URL").unwrap_or_else(|| DEFAULT_MSE_URL.to_string()),
            scrape_interval: Duration::from_secs(interval_secs),
            scraper_enabled,
            static_dir: get("STATIC_DIR")
                .unwrap_or_else(|| "web/dist".to_string())
                .into(),
        })
    }
}

fn parse_or<T>(
    get: &impl Fn(&str) -> Option<String>,
    key: &'static str,
    default: &str,
    parse: impl Fn(&str) -> Option<T>,
    reason: &'static str,
) -> Result<T, ConfigError> {
    let value = get(key).unwrap_or_else(|| default.to_string());
    parse(value.trim()).ok_or(ConfigError::Invalid { key, value, reason })
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| map.get(k).cloned()
    }

    #[test]
    fn requires_database_url() {
        assert_eq!(
            Config::from_lookup(lookup(&[])).unwrap_err(),
            ConfigError::Missing("DATABASE_URL")
        );
    }

    #[test]
    fn applies_defaults() {
        let cfg = Config::from_lookup(lookup(&[("DATABASE_URL", "postgres://x")])).unwrap();
        assert_eq!(cfg.bind_addr, "0.0.0.0:8080".parse().unwrap());
        assert_eq!(cfg.mse_url, DEFAULT_MSE_URL);
        assert_eq!(cfg.scrape_interval, Duration::from_secs(300));
        assert!(cfg.scraper_enabled);
        assert_eq!(cfg.static_dir, PathBuf::from("web/dist"));
    }

    #[test]
    fn reads_overrides() {
        let cfg = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            ("BIND_ADDR", "127.0.0.1:9000"),
            ("SCRAPE_INTERVAL_SECS", "60"),
            ("SCRAPER_ENABLED", "off"),
        ]))
        .unwrap();
        assert_eq!(cfg.bind_addr.port(), 9000);
        assert_eq!(cfg.scrape_interval, Duration::from_secs(60));
        assert!(!cfg.scraper_enabled);
    }

    #[test]
    fn rejects_bad_values() {
        for (key, value) in [
            ("BIND_ADDR", "nope"),
            ("SCRAPE_INTERVAL_SECS", "5"),
            ("SCRAPE_INTERVAL_SECS", "abc"),
            ("SCRAPER_ENABLED", "maybe"),
        ] {
            let err =
                Config::from_lookup(lookup(&[("DATABASE_URL", "postgres://x"), (key, value)]))
                    .unwrap_err();
            assert!(
                matches!(err, ConfigError::Invalid { key: k, .. } if k == key),
                "{key}={value}"
            );
        }
    }
}
