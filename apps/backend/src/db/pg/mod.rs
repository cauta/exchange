use anyhow::Context;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

/// Create a PostgreSQL connection pool
pub async fn create_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("PG_URL").context("PG_URL must be set in environment")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    Ok(pool)
}
