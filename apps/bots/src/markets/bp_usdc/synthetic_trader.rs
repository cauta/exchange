use crate::utils::bot_helpers;
use anyhow::Result;
use backend::models::domain::{Market, OrderType, Side};
use exchange_sdk::ExchangeClient;
use rand::{Rng, SeedableRng};
use std::time::Duration;
use tracing::{info, warn};

/// Configuration for the synthetic trader bot
#[derive(Clone, Debug)]
pub struct SyntheticTraderConfig {
    pub user_address: String,
    pub min_interval_ms: u64,     // Minimum time between trades
    pub max_interval_ms: u64,     // Maximum time between trades
    pub min_size: f64,            // Minimum trade size (BP)
    pub max_size: f64,            // Maximum trade size (BP)
    pub buy_probability: f64,     // Probability of buy vs sell [0.0, 1.0]
}

/// Synthetic Trader bot - generates realistic trading activity for prediction markets
///
/// Creates trades by hitting the LMSR market maker's orders:
/// - Random buy/sell decisions
/// - Random sizes and intervals
/// - Uses market orders to ensure execution
pub struct SyntheticTraderBot {
    config: SyntheticTraderConfig,
    exchange_client: ExchangeClient,
    market: Market,
}

impl SyntheticTraderBot {
    pub async fn new(
        config: SyntheticTraderConfig,
        exchange_client: ExchangeClient,
    ) -> Result<Self> {
        info!("Synthetic Trader bot initialized for BP/USDC");

        // Fetch market configuration and auto-faucet initial funds
        let market = bot_helpers::fetch_market_and_faucet(
            &exchange_client,
            "BP/USDC",
            &config.user_address,
        )
        .await?;

        info!(
            "Trade intervals: {}-{}ms, Size range: {}-{} BP",
            config.min_interval_ms, config.max_interval_ms, config.min_size, config.max_size
        );

        Ok(Self {
            config,
            exchange_client,
            market,
        })
    }

    /// Start the bot
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting synthetic trader for BP/USDC");

        let mut rng = rand::rngs::StdRng::from_entropy();

        loop {
            // Random interval between trades
            let interval_ms =
                rng.gen_range(self.config.min_interval_ms..=self.config.max_interval_ms);
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;

            // Decide buy or sell
            let side = if rng.gen::<f64>() < self.config.buy_probability {
                Side::Buy
            } else {
                Side::Sell
            };

            // Random size
            let size = rng.gen_range(self.config.min_size..=self.config.max_size);

            // Execute trade
            if let Err(e) = self.execute_trade(side.clone(), size).await {
                warn!("Failed to execute {:?} trade: {}", side, e);

                // Try to auto-faucet if balance issue
                bot_helpers::auto_faucet_on_error(
                    &self.exchange_client,
                    &self.config.user_address,
                    &self.market,
                    &e.to_string(),
                )
                .await;
            }
        }
    }

    /// Execute a market order
    async fn execute_trade(&self, side: Side, size: f64) -> Result<()> {
        // For market orders, we need to specify a price limit
        // Use a very favorable limit price to ensure execution:
        // - For buys: high limit (willing to pay up to $0.999)
        // - For sells: low limit (willing to sell down to $0.001)
        let limit_price = match side {
            Side::Buy => "0.999",   // Buy at any price up to $0.999
            Side::Sell => "0.001",  // Sell at any price down to $0.001
        };

        let result = self
            .exchange_client
            .place_order_decimal(
                self.config.user_address.clone(),
                "BP/USDC".to_string(),
                side.clone(),
                OrderType::Market,
                limit_price.to_string(),
                format!("{:.6}", size),
                "synthetic_trader".to_string(),
            )
            .await?;

        info!(
            "🎲 Synthetic trade executed: {:?} {:.2} BP (order: {})",
            side, size, result.order.id
        );

        Ok(())
    }
}
