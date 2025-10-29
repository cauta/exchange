use backend::db::Db;
use clickhouse::Client;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::{clickhouse::ClickHouse, postgres::Postgres};

/// Container handles for cleanup
pub struct TestContainers {
    pub(crate) db: Db,
    pub(crate) _postgres_container: testcontainers::ContainerAsync<Postgres>,
    pub(crate) _clickhouse_container: testcontainers::ContainerAsync<ClickHouse>,
}

impl TestContainers {
    /// Set up test databases with containers
    ///
    /// This starts PostgreSQL and ClickHouse containers, runs migrations,
    /// and returns handles. The containers will be cleaned up when dropped.
    pub async fn setup() -> anyhow::Result<Self> {
        // ================================ Start containers ================================
        let postgres_container = Postgres::default()
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start PostgreSQL container: {}", e))?;

        let clickhouse_container = ClickHouse::default()
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start ClickHouse container: {}", e))?;

        let postgres_port = postgres_container
            .get_host_port_ipv4(5432)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get PostgreSQL port: {}", e))?;

        let clickhouse_port = clickhouse_container
            .get_host_port_ipv4(8123)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get ClickHouse port: {}", e))?;

        // ================================ Create database connections ================================
        let postgres_url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            postgres_container.get_host().await.unwrap(),
            postgres_port
        );

        let clickhouse_url = format!(
            "http://{}:{}",
            clickhouse_container.get_host().await.unwrap(),
            clickhouse_port
        );

        let postgres = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&postgres_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to PostgreSQL: {}", e))?;

        // Create ClickHouse client without database first
        let clickhouse_temp = Client::default()
            .with_url(&clickhouse_url)
            .with_user("default");

        // ================================ Run migrations ================================
        sqlx::migrate!("../../apps/backend/src/db/pg/migrations")
            .run(&postgres)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to run PostgreSQL migrations: {}", e))?;

        // Initialize ClickHouse schema (creates database)
        Self::setup_clickhouse_schema(&clickhouse_temp).await?;

        // Now create client with the database set
        let clickhouse = Client::default()
            .with_url(&clickhouse_url)
            .with_user("default")
            .with_database("exchange");

        // ================================ Return database connections ================================
        let db = Db {
            postgres,
            clickhouse,
        };

        Ok(TestContainers {
            db,
            _postgres_container: postgres_container,
            _clickhouse_container: clickhouse_container,
        })
    }

    /// Set up ClickHouse schema for testing
    async fn setup_clickhouse_schema(client: &Client) -> anyhow::Result<()> {
        // Create database first
        client
            .query("CREATE DATABASE IF NOT EXISTS exchange")
            .execute()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create ClickHouse database: {}", e))?;

        // Load and execute schema from file
        const SCHEMA_SQL: &str = include_str!("../../../apps/backend/src/db/ch/schema.sql");

        // Remove comments and split by semicolon
        let sql_without_comments: String = SCHEMA_SQL
            .lines()
            .filter(|line| !line.trim().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");

        // Execute each statement
        for statement in sql_without_comments.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                client
                    .query(trimmed)
                    .execute()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to execute schema: {}", e))?;
            }
        }

        Ok(())
    }

    /// Get a clone of the database connection
    ///
    /// This is public to allow backend tests to access the database.
    /// Backend tests wrap this in their own TestDb struct.
    /// SDK tests should NOT use this - they should only test via HTTP API.
    pub fn db_clone(&self) -> Db {
        self.db.clone()
    }
}
