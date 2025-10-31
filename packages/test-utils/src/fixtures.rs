use crate::TestServer;
use exchange_sdk::ExchangeClient;

/// High-level test fixture for exchange testing
///
/// Provides a running test exchange with a pre-configured market and admin client.
/// Use this for SDK and integration tests that need a complete exchange setup.
pub struct TestExchange {
    pub server: TestServer,
    pub client: ExchangeClient,
    pub market_id: String,
    pub base_ticker: String,
    pub quote_ticker: String,
}

impl TestExchange {
    /// Create a new test exchange with BTC/USDC market
    pub async fn new() -> anyhow::Result<Self> {
        Self::with_market("BTC", "USDC").await
    }

    /// Create a test exchange with a custom market
    pub async fn with_market(base: &str, quote: &str) -> anyhow::Result<Self> {
        let server = TestServer::start().await?;
        let client = ExchangeClient::new(&server.base_url);

        // Setup tokens via admin API
        client
            .admin_create_token(base.to_string(), 18, format!("{} Token", base))
            .await?;
        client
            .admin_create_token(quote.to_string(), 18, format!("{} Token", quote))
            .await?;

        // Setup market via admin API
        let market = client
            .admin_create_market(
                base.to_string(),
                quote.to_string(),
                1000,    // tick_size
                1000000, // lot_size
                1000000, // min_size
                10,      // maker_fee_bps (0.1%)
                20,      // taker_fee_bps (0.2%)
            )
            .await?;

        Ok(Self {
            server,
            client,
            market_id: market.id,
            base_ticker: base.to_string(),
            quote_ticker: quote.to_string(),
        })
    }

    /// Create a user with starting balance
    ///
    /// Uses the admin faucet API to give users tokens, which also creates them if needed.
    pub async fn create_user_with_balance(
        &self,
        address: &str,
        base_amount: u128,
        quote_amount: u128,
    ) -> anyhow::Result<()> {
        // Use admin faucet to give tokens (this also creates user if needed)
        if base_amount > 0 {
            self.client
                .admin_faucet(
                    address.to_string(),
                    self.base_ticker.clone(),
                    base_amount.to_string(),
                )
                .await?;
        }
        if quote_amount > 0 {
            self.client
                .admin_faucet(
                    address.to_string(),
                    self.quote_ticker.clone(),
                    quote_amount.to_string(),
                )
                .await?;
        }

        Ok(())
    }
}

/// Helper to wait for a condition with timeout
pub async fn wait_for<F, Fut>(mut condition: F, timeout_ms: u64) -> anyhow::Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    while start.elapsed().as_millis() < timeout_ms as u128 {
        if condition().await {
            return Ok(());
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    anyhow::bail!("Timeout waiting for condition")
}
