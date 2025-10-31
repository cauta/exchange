use crate::hyperliquid::{HlMessage, HyperliquidClient};
use anyhow::Result;
use backend::models::domain::{Market, OrderType, Side};
use exchange_sdk::ExchangeClient;
use tracing::{error, info, warn};

/// Configuration for the trade mirror bot
#[derive(Clone)]
pub struct TradeMirrorConfig {
    pub market_id: String,    // e.g., "BTC/USDC"
    pub user_address: String, // Bot's wallet address
}

/// Trade mirror bot - creates realistic trading activity by copying Hyperliquid trades
pub struct TradeMirrorBot {
    config: TradeMirrorConfig,
    exchange_client: ExchangeClient,

    // Market configuration fetched from backend
    market: Market,
}

impl TradeMirrorBot {
    pub async fn new(config: TradeMirrorConfig, exchange_client: ExchangeClient) -> Result<Self> {
        // Fetch market configuration from backend
        let market = exchange_client.get_market(&config.market_id).await?;

        info!(
            "Trade mirror bot initialized for market {}",
            config.market_id
        );
        info!(
            "Market: {} (base) / {} (quote)",
            market.base_ticker, market.quote_ticker
        );

        // Auto-faucet initial funds (large amounts for testing)
        info!(
            "💰 Auto-fauceting initial funds for {}",
            config.user_address
        );
        let faucet_amount = "10000000000000000000000000";

        for token in [&market.base_ticker, &market.quote_ticker] {
            match exchange_client
                .admin_faucet(
                    config.user_address.clone(),
                    token.to_string(),
                    faucet_amount.to_string(),
                )
                .await
            {
                Ok(_) => info!("✓ Fauceted {} for {}", token, config.user_address),
                Err(e) => info!("Note: Faucet {} for {}: {}", token, config.user_address, e),
            }
        }

        Ok(Self {
            config,
            exchange_client,
            market,
        })
    }

    /// Start the bot
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting trade mirror bot for {} PERP -> {} market",
            self.market.base_ticker, self.config.market_id
        );

        // Connect to Hyperliquid (perps by default)
        let hl_client = HyperliquidClient::new(self.market.base_ticker.clone());

        let (mut rx, _handle) = hl_client.start().await?;

        // Process messages
        while let Some(msg) = rx.recv().await {
            match msg {
                HlMessage::L2Book(_) => {
                    // Trade bot doesn't care about orderbook
                }
                HlMessage::Trade(trades) => {
                    // Mirror each trade
                    for trade in trades {
                        if let Err(e) = self.mirror_trade(&trade.px, &trade.sz, &trade.side).await {
                            error!("Failed to mirror trade: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Mirror a Hyperliquid trade on our exchange
    async fn mirror_trade(&self, price_str: &str, size_str: &str, side_str: &str) -> Result<()> {
        // Determine side from Hyperliquid:
        // "A" = ask/sell (seller initiated), "B" = bid/buy (buyer initiated)
        let side = match side_str {
            "A" => Side::Sell,
            "B" => Side::Buy,
            _ => {
                warn!("Unknown trade side: {}", side_str);
                return Ok(());
            }
        };

        info!(
            "Mirroring {} trade: {:?} {} @ {}",
            self.market.base_ticker, side, size_str, price_str
        );

        // Place market order with human-readable decimal values
        // The SDK will handle conversion to atoms
        match self
            .exchange_client
            .place_order_decimal(
                self.config.user_address.clone(),
                self.config.market_id.clone(),
                side,
                OrderType::Market,
                price_str.to_string(),
                size_str.to_string(),
                "trade_mirror".to_string(),
            )
            .await
        {
            Ok(result) => {
                info!(
                    "Trade mirrored successfully: {} trades executed",
                    result.trades.len()
                );
            }
            Err(e) => {
                let err_msg = e.to_string();
                warn!("Failed to place trade mirror order: {}", err_msg);

                // Try to auto-faucet if it's a balance error
                self.auto_faucet_on_error(&err_msg).await;
            }
        }

        Ok(())
    }

    /// Auto-faucet funds if we detect insufficient balance error
    async fn auto_faucet_on_error(&self, error_msg: &str) -> bool {
        // Check if error is about insufficient balance
        if error_msg.contains("Insufficient balance") || error_msg.contains("insufficient") {
            // Extract token from error message if possible
            let token = if error_msg.contains("BTC") {
                Some("BTC")
            } else if error_msg.contains("USDC") {
                Some("USDC")
            } else {
                None
            };

            if let Some(token_name) = token {
                info!(
                    "💰 Detected insufficient {}, auto-fauceting more...",
                    token_name
                );
                let faucet_amount = "10000000000000000000000000";

                match self
                    .exchange_client
                    .admin_faucet(
                        self.config.user_address.clone(),
                        token_name.to_string(),
                        faucet_amount.to_string(),
                    )
                    .await
                {
                    Ok(_) => {
                        info!(
                            "✓ Auto-fauceted {} for {}",
                            token_name, self.config.user_address
                        );
                        return true;
                    }
                    Err(e) => {
                        warn!("Failed to auto-faucet {}: {}", token_name, e);
                    }
                }
            }
        }
        false
    }
}
