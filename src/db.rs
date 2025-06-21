use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn init_db_pool() -> Result<PgPool, sqlx::Error> {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL harus diatur di .env");

    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
}