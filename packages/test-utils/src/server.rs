use crate::db::TestDb;
use crate::engine::TestEngine;
use axum::Router;
use backend::api::{rest, ws};
use backend::db::Db;
use backend::AppState;
use tower_http::cors::CorsLayer;

/// Handle to a running test server
///
/// Provides both simple URL-based testing (for SDK) and internal access to DB/engine (for backend tests)
#[allow(dead_code)]
pub struct TestServer {
    pub base_url: String,
    pub ws_url: String,
    /// Alias for base_url for backwards compatibility with backend tests
    pub address: String,
    pub test_db: TestDb,
    pub test_engine: TestEngine,
    _shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

#[allow(dead_code)]
impl TestServer {
    /// Start a test HTTP server on a random available port
    ///
    /// This spawns:
    /// - PostgreSQL and ClickHouse containers
    /// - Matching engine
    /// - Axum HTTP server with REST + WebSocket routes
    ///
    /// The server runs in the background and will shutdown when dropped.
    pub async fn start() -> anyhow::Result<Self> {
        // Setup database
        let test_db = TestDb::setup().await?;

        // Setup matching engine using TestEngine (without creating users)
        // Integration tests will create their own users
        let test_engine = TestEngine::new_with_users(&test_db, false).await;

        // Create REST and WebSocket routes
        let rest = rest::create_rest();
        let ws = ws::create_ws();
        let state = AppState {
            db: test_engine.db.clone(),
            engine_tx: test_engine.engine_tx.clone(),
            event_tx: test_engine.event_tx(),
        };
        let app = Router::new()
            .merge(rest)
            .merge(ws)
            .with_state(state)
            .layer(CorsLayer::permissive());

        // Bind to random available port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind test server: {}", e))?;

        let addr = listener
            .local_addr()
            .map_err(|e| anyhow::anyhow!("Failed to get local address: {}", e))?;

        let base_url = format!("http://{}", addr);
        let ws_url = format!("ws://{}/ws", addr);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn server in background
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .expect("Server failed to start");
        });

        // Give server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(Self {
            base_url: base_url.clone(),
            ws_url,
            address: base_url, // Alias for backwards compatibility
            test_db,
            test_engine,
            _shutdown_tx: shutdown_tx,
        })
    }

    // ============================================================================
    // URL Builders - Use these with raw reqwest/tokio-tungstenite in tests
    // ============================================================================

    /// Build full HTTP URL for a path
    ///
    /// # Example
    /// ```no_run
    /// # use exchange_test_utils::TestServer;
    /// # async fn example(server: &TestServer) -> Result<(), Box<dyn std::error::Error>> {
    /// let url = server.url("/api/health");
    /// let response = reqwest::get(&url).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }


    // ============================================================================
    // Database Access - Backend tests can access internals
    // ============================================================================

    /// Get reference to database connection for direct DB operations in tests
    pub fn db(&self) -> &Db {
        &self.test_db.db
    }

    /// Get reference to test engine for engine operations in tests
    pub fn engine(&self) -> &TestEngine {
        &self.test_engine
    }
}
