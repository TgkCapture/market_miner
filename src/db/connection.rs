use tokio_postgres::{NoTls, Client, Error};

pub async fn get_db_connection() -> Result<Client, Error> {
    let (client, connection) =
        tokio_postgres::connect("host={} user={} password={} port={} dbname=postgres", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    Ok(client)
}
