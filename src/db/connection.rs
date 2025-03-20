// db/connection.rs
use tokio_postgres::{NoTls, Error};
use dotenvy::dotenv;
use std::env;
use crate::utils::logging::{log_info};

fn get_db_config() -> (String, String, String, String, String) {
    dotenv().ok(); // Load .env file

    let host = env::var("DB_HOST").expect("DB_HOST must be set");
    let user = env::var("DB_USER").expect("DB_USER must be set");
    let password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let dbname = env::var("DB_NAME").expect("DB_NAME must be set");
    let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());

    (host, user, password, dbname, port)
}

/// Creates the database if it does not exist
pub async fn create_database_if_not_exists() -> Result<(), Error> {
    let (host, user, password, dbname, port) = get_db_config();
    
    let admin_conn_str = format!("host={} user={} password={} port={} dbname=postgres", host, user, password, port);
    
    let (client, connection) = tokio_postgres::connect(&admin_conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    let check_db_query = "SELECT 1 FROM pg_database WHERE datname = $1";
    let exists = client.query(check_db_query, &[&dbname]).await?;

    if exists.is_empty() {
        println!("Database '{}' does not exist. Creating...", dbname);
        client
            .execute(&format!("CREATE DATABASE {}", dbname), &[])
            .await?;
    } else {
        log_info(&format!("Database '{}' already exists.", dbname));
    }

    Ok(())
}

/// Connects to the PostgreSQL database
pub async fn connect_db() -> Result<tokio_postgres::Client, Error> {
    let (host, user, password, dbname, port) = get_db_config();
    let conn_str = format!("host={} user={} password={} dbname={} port={}", host, user, password, dbname, port);

    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    // Ensure the stocks table exists
    client.execute(
        "CREATE TABLE IF NOT EXISTS stocks (
            id SERIAL PRIMARY KEY,
            symbol TEXT NOT NULL,
            open_price DOUBLE PRECISION,
            close_price DOUBLE PRECISION,
            percent_change DOUBLE PRECISION,
            volume DOUBLE PRECISION,
            turnover DOUBLE PRECISION,
            timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )",
        &[],
    ).await?;

    Ok(client)
}