// Database modules
pub mod ch;
pub mod pg;

// Re-export common types
pub use clickhouse::Client;
pub use sqlx::postgres::PgPool;

/// Main database handle with connections to both databases
pub struct Db {
    pub postgres: PgPool,
    pub clickhouse: Client,
}

impl Db {
    /// Create a new Db instance with connections to both databases
    pub async fn connect() -> anyhow::Result<Self> {
        let postgres = pg::create_pool().await?;
        let clickhouse = ch::create_client().await?;

        Ok(Self {
            postgres,
            clickhouse,
        })
    }
}
