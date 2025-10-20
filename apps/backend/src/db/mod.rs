// Database modules
pub mod postgres;
pub mod clickhouse;

// Re-export common types
pub use sqlx::postgres::PgPool;
pub use clickhouse::Client;

/// Main database handle with connections to both databases
pub struct Db {
    pub postgres: PgPool,
    pub clickhouse: Client,
}

impl Db {
    /// Create a new Db instance with connections to both databases
    pub async fn connect() -> anyhow::Result<Self> {
        let postgres = postgres::create_pool().await?;
        let clickhouse = clickhouse::create_client();

        Ok(Self {
            postgres,
            clickhouse,
        })
    }
}
