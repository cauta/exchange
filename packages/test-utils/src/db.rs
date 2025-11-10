use backend::db::Db;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::{clickhouse::ClickHouse, postgres::Postgres};

// ============================================================================
// Test Containers - Database Setup
// ============================================================================

/// Container handles for test databases
pub struct TestContainers {
    pub(crate) db: Db,
    pub(crate) _postgres_container: testcontainers::ContainerAsync<Postgres>,
    pub(crate) _clickhouse_container: testcontainers::ContainerAsync<ClickHouse>,
}

impl TestContainers {
    /// Set up test databases with containers
    ///
    /// This starts PostgreSQL and ClickHouse containers, sets environment variables,
    /// and uses Db::connect() to automatically run migrations.
    /// The containers will be cleaned up when dropped.
    pub async fn setup() -> anyhow::Result<Self> {
        // Start containers
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

        // Build connection URLs
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

        // Connect with explicit URLs to avoid conflicts in parallel tests
        let db = Db::connect_with_urls(Some(postgres_url), Some(clickhouse_url))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to databases: {}", e))?;

        Ok(TestContainers {
            db,
            _postgres_container: postgres_container,
            _clickhouse_container: clickhouse_container,
        })
    }

    /// Get a clone of the database connection
    pub fn db_clone(&self) -> Db {
        self.db.clone()
    }
}

// ============================================================================
// Test Database - Wrapper with Direct DB Access
// ============================================================================

/// Test database wrapper
///
/// Provides access to the database for backend tests that need to verify internal state.
/// For e2e tests, prefer using HTTP API helpers instead.
#[allow(dead_code)]
pub struct TestDb {
    pub db: Db,
    _containers: TestContainers,
}

#[allow(dead_code)]
impl TestDb {
    /// Set up test databases with containers
    pub async fn setup() -> anyhow::Result<Self> {
        let containers = TestContainers::setup().await?;
        let db = containers.db_clone();

        Ok(TestDb {
            db,
            _containers: containers,
        })
    }
}
