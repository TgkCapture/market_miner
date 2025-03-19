pub use tokio_postgres::Client;
pub mod connection;
pub use connection::{create_database_if_not_exists, connect_db};

pub mod queries;
pub use queries::insert_stock_data;
