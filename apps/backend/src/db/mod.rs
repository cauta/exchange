// Database connection modules
pub mod ch;
pub mod pg;

pub mod balances;
pub mod candles;
pub mod markets;
pub mod orders;
pub mod tokens;
pub mod trades;
pub mod users;

// Re-export common types
pub use clickhouse::Client;
pub use sqlx::postgres::{PgPool, Postgres};
pub use sqlx::Transaction;

/// Main database handle with connections to both databases
#[derive(Clone)]
pub struct Db {
    pub postgres: PgPool,
    pub clickhouse: Client,
}

impl Db {
    /// Create a new Db instance with connections to both databases
    /// Uses environment variables PG_URL and CH_URL
    pub async fn connect() -> anyhow::Result<Self> {
        Self::connect_with_urls(None, None).await
    }

    /// Create a new Db instance with explicit URLs
    /// Useful for testing to avoid environment variable conflicts
    pub async fn connect_with_urls(
        pg_url: Option<String>,
        ch_url: Option<String>,
    ) -> anyhow::Result<Self> {
        let postgres = pg::create_pool(pg_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create PostgreSQL pool: {}", e))?;

        let clickhouse = ch::create_client(ch_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create ClickHouse client: {}", e))?;

        Ok(Self {
            postgres,
            clickhouse,
        })
    }

    /// Begin a new database transaction
    pub async fn begin_transaction(&self) -> crate::errors::Result<Transaction<'_, Postgres>> {
        Ok(self.postgres.begin().await?)
    }
}
