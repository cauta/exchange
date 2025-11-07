use super::hyperliquid::{HlMessage, HyperliquidClient};
use crate::utils::bot_helpers;
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
        info!(
            "Trade mirror bot initialized for market {}",
            config.market_id
        );

        // Fetch market configuration and auto-faucet initial funds
        let market = bot_helpers::fetch_market_and_faucet(
            &exchange_client,
            &config.market_id,
            &config.user_address,
        )
        .await?;

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
        bot_helpers::auto_faucet_on_error(
            &self.exchange_client,
            &self.config.user_address,
            &self.market,
            error_msg,
        )
        .await
    }
}
