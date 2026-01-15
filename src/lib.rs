pub mod domain;
pub mod handlers;
pub mod repositories;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub async fn establish_connection(database_url: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to connect to Postgres")
}
