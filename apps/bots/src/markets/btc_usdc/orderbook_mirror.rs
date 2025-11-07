use super::hyperliquid::{HlMessage, HyperliquidClient, Orderbook};
use crate::utils::bot_helpers;
use anyhow::Result;
use backend::models::domain::{Market, OrderType, Side};
use exchange_sdk::ExchangeClient;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Configuration for the orderbook mirror bot
#[derive(Clone)]
pub struct OrderbookMirrorConfig {
    pub market_id: String,       // e.g., "BTC/USDC"
    pub user_address: String,    // Bot's wallet address
    pub depth_levels: usize,     // How many levels to mirror (e.g., 5)
    pub update_interval_ms: u64, // Min time between order updates
}

/// Orderbook mirror bot - maintains liquidity by copying Hyperliquid's orderbook
pub struct OrderbookMirrorBot {
    config: OrderbookMirrorConfig,
    exchange_client: ExchangeClient,
    orderbook: Orderbook,
    active_orders: HashMap<String, Uuid>, // price_side -> order_id

    // Market configuration fetched from backend
    market: Market,
}

impl OrderbookMirrorBot {
    pub async fn new(
        config: OrderbookMirrorConfig,
        exchange_client: ExchangeClient,
    ) -> Result<Self> {
        info!(
            "Orderbook mirror bot initialized for market {}",
            config.market_id
        );

        // Fetch market configuration and auto-faucet initial funds
        let market = bot_helpers::fetch_market_and_faucet(
            &exchange_client,
            &config.market_id,
            &config.user_address,
        )
        .await?;

        // Use base ticker as coin symbol for Hyperliquid (e.g., "BTC" from "BTC/USDC")
        let coin = market.base_ticker.clone();
        let orderbook = Orderbook::new(coin.clone());

        Ok(Self {
            config,
            exchange_client,
            orderbook,
            active_orders: HashMap::new(),
            market,
        })
    }

    /// Start the bot
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting orderbook mirror bot for {} PERP -> {} market",
            self.market.base_ticker, self.config.market_id
        );
        info!(
            "Update interval: {}ms (throttling to prevent spam)",
            self.config.update_interval_ms
        );

        // Cancel all existing orders on startup to ensure clean state
        info!("Cancelling any existing orders from previous runs...");
        self.cancel_all_orders().await?;

        // Connect to Hyperliquid (perps by default)
        let hl_client = HyperliquidClient::new(self.market.base_ticker.clone());

        let (mut rx, _handle) = hl_client.start().await?;

        // Throttling: track last update time
        let mut last_sync = Instant::now();
        let update_interval = Duration::from_millis(self.config.update_interval_ms);

        // Process messages
        while let Some(msg) = rx.recv().await {
            match msg {
                HlMessage::L2Book(book_data) => {
                    // Update local orderbook snapshot (fast, in-memory)
                    if book_data.levels.len() >= 2 {
                        let bids = book_data.levels[0].clone();
                        let asks = book_data.levels[1].clone();
                        self.orderbook.update_from_l2(bids, asks);

                        // Only sync with exchange if enough time has passed (throttling)
                        let now = Instant::now();
                        if now.duration_since(last_sync) >= update_interval {
                            if let Err(e) = self.sync_orderbook().await {
                                error!("Failed to sync orderbook: {}", e);
                            }
                            last_sync = now;
                        }
                    }
                }
                HlMessage::Trade(_) => {
                    // Orderbook bot doesn't care about trades
                }
            }
        }

        Ok(())
    }

    /// Sync our exchange's orderbook with Hyperliquid
    async fn sync_orderbook(&mut self) -> Result<()> {
        let (bids, asks) = self.orderbook.get_top_levels(self.config.depth_levels);

        // Separate old orders by side to prevent crossing
        let mut old_bids = Vec::new();
        let mut old_asks = Vec::new();

        for (key, order_id) in self.active_orders.drain() {
            if key.ends_with("_buy") {
                old_bids.push(order_id);
            } else {
                old_asks.push(order_id);
            }
        }

        // Strategy to avoid crossing while maintaining liquidity:
        // 1. Cancel old asks first (keeps bid side liquid)
        // 2. Place new asks (now we have bids + new asks)
        // 3. Cancel old bids (keeps ask side liquid)
        // 4. Place new bids (now we have complete new book)

        // Step 1: Cancel old asks
        self.cancel_orders_list(old_asks).await;

        // Step 2: Place new ask orders
        for level in asks {
            let price = level.price.to_string();
            let size = level.quantity.to_string();

            match self
                .exchange_client
                .place_order_decimal(
                    self.config.user_address.clone(),
                    self.config.market_id.clone(),
                    Side::Sell,
                    OrderType::Limit,
                    price.clone(),
                    size.clone(),
                    "orderbook_mirror".to_string(),
                )
                .await
            {
                Ok(result) => {
                    let order_id = result.order.id;
                    let key = format!("{}_{}", price, "sell");
                    self.active_orders.insert(key, order_id);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    warn!("Failed to place ask order at {}: {}", price, err_msg);

                    // Try to auto-faucet if it's a balance error
                    self.auto_faucet_on_error(&err_msg).await;
                }
            }
        }

        // Step 3: Cancel old bids
        self.cancel_orders_list(old_bids).await;

        // Step 4: Place new bid orders
        for level in bids {
            let price = level.price.to_string();
            let size = level.quantity.to_string();

            match self
                .exchange_client
                .place_order_decimal(
                    self.config.user_address.clone(),
                    self.config.market_id.clone(),
                    Side::Buy,
                    OrderType::Limit,
                    price.clone(),
                    size.clone(),
                    "orderbook_mirror".to_string(),
                )
                .await
            {
                Ok(result) => {
                    let order_id = result.order.id;
                    let key = format!("{}_{}", price, "buy");
                    self.active_orders.insert(key, order_id);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    warn!("Failed to place bid order at {}: {}", price, err_msg);

                    // Try to auto-faucet if it's a balance error
                    self.auto_faucet_on_error(&err_msg).await;
                }
            }
        }

        Ok(())
    }

    /// Cancel a list of orders by ID
    async fn cancel_orders_list(&self, order_ids: Vec<Uuid>) {
        if order_ids.is_empty() {
            return;
        }

        let mut cancelled_count = 0;
        for order_id in order_ids {
            match self
                .exchange_client
                .cancel_order(
                    self.config.user_address.clone(),
                    order_id.to_string(),
                    "orderbook_mirror".to_string(),
                )
                .await
            {
                Ok(_) => {
                    cancelled_count += 1;
                }
                Err(e) => {
                    // It's okay if order is already filled/cancelled
                    warn!("Failed to cancel order {}: {}", order_id, e);
                }
            }
        }

        if cancelled_count > 0 {
            info!("Cancelled {} orders", cancelled_count);
        }
    }

    /// Cancel all active orders
    async fn cancel_all_orders(&mut self) -> Result<()> {
        // Use the new cancel_all_orders endpoint for efficient bulk cancellation
        match self
            .exchange_client
            .cancel_all_orders(
                self.config.user_address.clone(),
                Some(self.config.market_id.clone()),
                "orderbook_mirror".to_string(),
            )
            .await
        {
            Ok(result) => {
                info!(
                    "Cancelled {} orders for market {}",
                    result.count, self.config.market_id
                );
            }
            Err(e) => {
                warn!("Failed to cancel all orders: {}", e);
            }
        }

        self.active_orders.clear();
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
