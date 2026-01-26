//! Adapter for managing multiple OrderBook-rs instances across markets
//!
//! Uses BookManagerTokio for async-compatible trade event processing.
//! The architecture maintains:
//! - Lock-free OrderBook operations via crossbeam-skiplist
//! - O(1) order cancellation via UUID → market index
//! - Centralized trade event routing through TradeListener

use chrono::Utc;
use dashmap::DashMap;
use log::{debug, info};
use orderbook_rs::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::errors::{ExchangeError, Result};
use crate::models::domain::{
    EngineEvent, Market, Order, OrderStatus, OrderbookLevel, OrderbookSnapshot, OrderbookStats,
    Side, Trade,
};

use super::price_converter::PriceConverter;

/// Metadata stored alongside each order in OrderBook-rs
#[derive(Clone, Debug)]
pub struct OrderMetadata {
    pub uuid: Uuid,
    pub user_address: String,
    pub market_id: String,
    pub original_price: u128,
    pub original_size: u128,
    pub filled_size: u128,
    pub side: Side,
    pub created_at: chrono::DateTime<Utc>,
}

impl Default for OrderMetadata {
    fn default() -> Self {
        Self {
            uuid: Uuid::nil(),
            user_address: String::new(),
            market_id: String::new(),
            original_price: 0,
            original_size: 0,
            filled_size: 0,
            side: Side::Buy, // Default to Buy
            created_at: Utc::now(),
        }
    }
}

/// Market configuration stored for each orderbook
#[derive(Clone, Debug)]
struct MarketConfig {
    #[allow(dead_code)]
    tick_size: u128,
    #[allow(dead_code)]
    lot_size: u128,
    converter: PriceConverter,
}

/// Manages multiple OrderBook-rs instances across markets
///
/// This adapter uses BookManagerTokio from orderbook-rs for:
/// - Managing multiple lock-free orderbooks
/// - Centralized trade event routing via TradeListener
/// - Async-compatible trade processing
///
/// Additional features:
/// - O(1) order cancellation via UUID → market index
/// - Domain type conversions (u128 ↔ u64)
/// - Orderbook snapshots with analytics
pub struct BookManagerAdapter {
    /// BookManagerTokio from orderbook-rs for managing all orderbooks
    manager: Arc<RwLock<BookManagerTokio<OrderMetadata>>>,
    /// Market configurations (tick_size, lot_size) per market
    market_configs: Arc<DashMap<String, MarketConfig>>,
    /// Global index for O(1) order cancellation: UUID → market_id
    uuid_to_market: Arc<DashMap<Uuid, String>>,
    /// Domain order storage for snapshot generation and order retrieval
    /// Key: market_id → (order_uuid → Order)
    order_storage: Arc<DashMap<String, HashMap<Uuid, Order>>>,
    /// Broadcast sender for engine events (trades, etc.)
    event_tx: Option<broadcast::Sender<EngineEvent>>,
}

impl Default for BookManagerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BookManagerAdapter {
    /// Create a new BookManagerAdapter
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(BookManagerTokio::new())),
            market_configs: Arc::new(DashMap::new()),
            uuid_to_market: Arc::new(DashMap::new()),
            order_storage: Arc::new(DashMap::new()),
            event_tx: None,
        }
    }

    /// Create a new BookManagerAdapter with event broadcasting
    pub fn with_event_tx(event_tx: broadcast::Sender<EngineEvent>) -> Self {
        Self {
            manager: Arc::new(RwLock::new(BookManagerTokio::new())),
            market_configs: Arc::new(DashMap::new()),
            uuid_to_market: Arc::new(DashMap::new()),
            order_storage: Arc::new(DashMap::new()),
            event_tx: Some(event_tx),
        }
    }

    /// Initialize a market's orderbook
    ///
    /// Must be called before adding orders for a market.
    pub async fn init_market(&self, market: &Market) {
        let mut manager = self.manager.write().await;

        if !manager.has_book(&market.id) {
            manager.add_book(&market.id);
            info!("Initialized orderbook for market: {}", market.id);
        }

        // Store market configuration
        self.market_configs.insert(
            market.id.clone(),
            MarketConfig {
                tick_size: market.tick_size,
                lot_size: market.lot_size,
                converter: PriceConverter::new(market.tick_size, market.lot_size),
            },
        );

        // Initialize order storage for this market
        self.order_storage
            .entry(market.id.clone())
            .or_insert_with(HashMap::new);
    }

    /// Add an order to the appropriate market
    ///
    /// The order is added to both the OrderBook-rs book and our domain storage.
    pub async fn add_order(&self, market: &Market, order: &Order) {
        // Ensure market is initialized
        self.init_market(market).await;

        let config = self
            .market_configs
            .get(&market.id)
            .expect("Market config should exist after init");

        let remaining_size = order.size.saturating_sub(order.filled_size);
        if remaining_size == 0 {
            return;
        }

        // Convert to OrderBook-rs types
        let ob_price = config.converter.price_to_ticks(order.price);
        let ob_size = config.converter.size_to_lots(remaining_size);
        let ob_side = match order.side {
            Side::Buy => orderbook_rs::Side::Buy,
            Side::Sell => orderbook_rs::Side::Sell,
        };

        // Create metadata for OrderBook-rs
        let metadata = OrderMetadata {
            uuid: order.id,
            user_address: order.user_address.clone(),
            market_id: market.id.clone(),
            original_price: order.price,
            original_size: order.size,
            filled_size: order.filled_size,
            side: order.side,
            created_at: order.created_at,
        };

        // Index UUID → market for O(1) cancellation
        self.uuid_to_market.insert(order.id, market.id.clone());

        // Store in domain order storage
        self.order_storage
            .entry(market.id.clone())
            .or_insert_with(HashMap::new)
            .insert(order.id, order.clone());

        // Add to OrderBook-rs
        let mut manager = self.manager.write().await;
        if let Some(book) = manager.get_book_mut(&market.id) {
            let order_id = OrderId::from_uuid(order.id);
            let result = book.add_limit_order(
                order_id,
                ob_price,
                ob_size,
                ob_side,
                TimeInForce::Gtc,
                Some(metadata),
            );

            debug!(
                "Added order {} to {}: price={} size={} side={:?} result={:?}",
                order.id, market.id, ob_price, ob_size, ob_side, result
            );
        }
    }

    /// Cancel an order using O(1) UUID lookup
    ///
    /// Returns the cancelled order if found and ownership is verified.
    pub async fn cancel_order(&self, order_id: Uuid, user_address: &str) -> Result<Order> {
        // O(1) lookup to find market
        let market_id = self
            .uuid_to_market
            .get(&order_id)
            .ok_or(ExchangeError::OrderNotFound)?
            .clone();

        // Get order from domain storage
        let order = {
            let mut market_orders = self
                .order_storage
                .get_mut(&market_id)
                .ok_or(ExchangeError::OrderNotFound)?;

            let order = market_orders
                .get(&order_id)
                .ok_or(ExchangeError::OrderNotFound)?
                .clone();

            // Verify ownership
            if order.user_address != user_address {
                return Err(ExchangeError::OrderNotFound);
            }

            // Remove from storage
            market_orders.remove(&order_id);
            order
        };

        // Remove from OrderBook-rs
        let mut manager = self.manager.write().await;
        if let Some(book) = manager.get_book_mut(&market_id) {
            let ob_order_id = OrderId::from_uuid(order_id);
            let _ = book.cancel_order(ob_order_id);
        }

        // Remove from UUID index
        self.uuid_to_market.remove(&order_id);

        debug!("Cancelled order {} in market {}", order_id, market_id);
        Ok(order)
    }

    /// Cancel all orders for a user, optionally filtered by market
    pub async fn cancel_all_orders(
        &self,
        user_address: &str,
        market_id: Option<&str>,
    ) -> Vec<Order> {
        let mut cancelled = Vec::new();
        let mut manager = self.manager.write().await;

        let markets_to_check: Vec<String> = if let Some(mid) = market_id {
            vec![mid.to_string()]
        } else {
            self.order_storage.iter().map(|e| e.key().clone()).collect()
        };

        for mid in markets_to_check {
            if let Some(mut market_orders) = self.order_storage.get_mut(&mid) {
                let user_order_ids: Vec<Uuid> = market_orders
                    .iter()
                    .filter(|(_, o)| o.user_address == user_address)
                    .map(|(id, _)| *id)
                    .collect();

                for order_id in user_order_ids {
                    if let Some(order) = market_orders.remove(&order_id) {
                        // Remove from OrderBook-rs
                        if let Some(book) = manager.get_book_mut(&mid) {
                            let ob_order_id = OrderId::from_uuid(order_id);
                            let _ = book.cancel_order(ob_order_id);
                        }

                        // Remove from UUID index
                        self.uuid_to_market.remove(&order_id);
                        cancelled.push(order);
                    }
                }
            }
        }

        debug!(
            "Cancelled {} orders for user {}",
            cancelled.len(),
            user_address
        );
        cancelled
    }

    /// Match a taker order against the orderbook
    ///
    /// Returns matches and emits trade events if configured.
    pub async fn match_order(
        &self,
        market: &Market,
        taker_order: &Order,
    ) -> Vec<crate::models::domain::Match> {
        use crate::models::domain::Match;

        let _config = match self.market_configs.get(&market.id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut matches = Vec::new();
        let mut remaining_size = taker_order.size.saturating_sub(taker_order.filled_size);

        // Get orders on the opposite side from our domain storage
        let opposite_orders: Vec<Order> = {
            let market_orders = match self.order_storage.get(&market.id) {
                Some(o) => o,
                None => return Vec::new(),
            };

            let mut orders: Vec<Order> = market_orders
                .values()
                .filter(|o| match taker_order.side {
                    Side::Buy => o.side == Side::Sell,
                    Side::Sell => o.side == Side::Buy,
                })
                .filter(|o| o.size > o.filled_size)
                .cloned()
                .collect();

            // Sort by price-time priority
            match taker_order.side {
                Side::Buy => {
                    orders.sort_by(|a, b| {
                        a.price
                            .cmp(&b.price)
                            .then_with(|| a.created_at.cmp(&b.created_at))
                    });
                }
                Side::Sell => {
                    orders.sort_by(|a, b| {
                        b.price
                            .cmp(&a.price)
                            .then_with(|| a.created_at.cmp(&b.created_at))
                    });
                }
            }
            orders
        };

        for maker_order in opposite_orders {
            if remaining_size == 0 {
                break;
            }

            // Check price compatibility
            let can_match = match (taker_order.side, taker_order.order_type) {
                (Side::Buy, crate::models::domain::OrderType::Limit) => {
                    taker_order.price >= maker_order.price
                }
                (Side::Buy, crate::models::domain::OrderType::Market) => true,
                (Side::Sell, crate::models::domain::OrderType::Limit) => {
                    taker_order.price <= maker_order.price
                }
                (Side::Sell, crate::models::domain::OrderType::Market) => true,
            };

            if !can_match {
                break;
            }

            // Skip self-trading
            if maker_order.user_address == taker_order.user_address {
                continue;
            }

            let maker_remaining = maker_order.size.saturating_sub(maker_order.filled_size);
            let match_size = remaining_size.min(maker_remaining);

            matches.push(Match {
                maker_order: maker_order.clone(),
                price: maker_order.price,
                size: match_size,
            });

            // Emit trade event
            if let Some(event_tx) = &self.event_tx {
                let (buyer_address, seller_address, buyer_order_id, seller_order_id) =
                    match taker_order.side {
                        Side::Buy => (
                            taker_order.user_address.clone(),
                            maker_order.user_address.clone(),
                            taker_order.id,
                            maker_order.id,
                        ),
                        Side::Sell => (
                            maker_order.user_address.clone(),
                            taker_order.user_address.clone(),
                            maker_order.id,
                            taker_order.id,
                        ),
                    };

                let trade = Trade {
                    id: Uuid::new_v4(),
                    market_id: market.id.clone(),
                    buyer_address,
                    seller_address,
                    buyer_order_id,
                    seller_order_id,
                    price: maker_order.price,
                    size: match_size,
                    side: taker_order.side,
                    timestamp: Utc::now(),
                };

                let _ = event_tx.send(EngineEvent::TradeExecuted { trade });
            }

            remaining_size -= match_size;
        }

        // Update maker orders' fill amounts
        for m in &matches {
            self.update_order_fill(m.maker_order.id, m.size).await;
        }

        matches
    }

    /// Update an order's filled amount
    pub async fn update_order_fill(&self, order_id: Uuid, fill_size: u128) {
        let market_id = match self.uuid_to_market.get(&order_id) {
            Some(m) => m.clone(),
            None => return,
        };

        if let Some(mut market_orders) = self.order_storage.get_mut(&market_id) {
            if let Some(order) = market_orders.get_mut(&order_id) {
                order.filled_size += fill_size;
                order.updated_at = Utc::now();

                if order.filled_size >= order.size {
                    order.status = OrderStatus::Filled;
                    // Remove fully filled order
                    market_orders.remove(&order_id);
                    self.uuid_to_market.remove(&order_id);

                    // Also remove from OrderBook-rs
                    let mut manager = self.manager.write().await;
                    if let Some(book) = manager.get_book_mut(&market_id) {
                        let ob_order_id = OrderId::from_uuid(order_id);
                        let _ = book.cancel_order(ob_order_id);
                    }
                }
            }
        }
    }

    /// Apply executed trades to the orderbook
    pub async fn apply_trades(&self, taker_order: &Order, trades: &[Trade], market: &Market) {
        // Update maker orders that were filled
        for trade in trades {
            let maker_order_id = match taker_order.side {
                Side::Buy => trade.seller_order_id,
                Side::Sell => trade.buyer_order_id,
            };
            self.update_order_fill(maker_order_id, trade.size).await;
        }

        // Add taker order to book if not fully filled
        let total_matched: u128 = trades.iter().map(|t| t.size).sum();
        let remaining_size = taker_order.size.saturating_sub(total_matched);

        if remaining_size > 0 && remaining_size >= market.min_size {
            let mut remaining_order = taker_order.clone();
            remaining_order.filled_size = total_matched;
            remaining_order.status = if total_matched > 0 {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Pending
            };
            self.add_order(market, &remaining_order).await;
        }
    }

    /// Generate snapshots for all markets
    pub fn snapshots(&self) -> Vec<OrderbookSnapshot> {
        self.order_storage
            .iter()
            .map(|entry| self.snapshot_for_market(entry.key()))
            .collect()
    }

    /// Generate a snapshot for a specific market
    pub fn snapshot_for_market(&self, market_id: &str) -> OrderbookSnapshot {
        let market_orders = match self.order_storage.get(market_id) {
            Some(o) => o,
            None => {
                return OrderbookSnapshot {
                    market_id: market_id.to_string(),
                    bids: Vec::new(),
                    asks: Vec::new(),
                    timestamp: Utc::now(),
                    stats: None,
                };
            }
        };

        // Aggregate bids by price level
        let mut bid_levels: HashMap<u128, u128> = HashMap::new();
        for order in market_orders.values().filter(|o| o.side == Side::Buy) {
            let remaining = order.size.saturating_sub(order.filled_size);
            *bid_levels.entry(order.price).or_insert(0) += remaining;
        }

        let mut bids: Vec<OrderbookLevel> = bid_levels
            .into_iter()
            .filter(|(_, size)| *size > 0)
            .map(|(price, size)| OrderbookLevel { price, size })
            .collect();
        bids.sort_by(|a, b| b.price.cmp(&a.price));

        // Aggregate asks by price level
        let mut ask_levels: HashMap<u128, u128> = HashMap::new();
        for order in market_orders.values().filter(|o| o.side == Side::Sell) {
            let remaining = order.size.saturating_sub(order.filled_size);
            *ask_levels.entry(order.price).or_insert(0) += remaining;
        }

        let mut asks: Vec<OrderbookLevel> = ask_levels
            .into_iter()
            .filter(|(_, size)| *size > 0)
            .map(|(price, size)| OrderbookLevel { price, size })
            .collect();
        asks.sort_by(|a, b| a.price.cmp(&b.price));

        OrderbookSnapshot {
            market_id: market_id.to_string(),
            bids,
            asks,
            timestamp: Utc::now(),
            stats: None,
        }
    }

    /// Generate enriched snapshots with analytics from OrderBook-rs
    pub async fn enriched_snapshots(&self) -> Vec<OrderbookSnapshot> {
        let mut snapshots = Vec::new();
        let manager = self.manager.read().await;

        for entry in self.order_storage.iter() {
            let market_id = entry.key();
            let mut snapshot = self.snapshot_for_market(market_id);

            // Get analytics from OrderBook-rs if available
            if let Some(book) = manager.get_book(market_id) {
                if let Some(config) = self.market_configs.get(market_id) {
                    let enriched = book.enriched_snapshot(100);

                    let spread = match (book.best_bid(), book.best_ask()) {
                        (Some(bid), Some(ask)) => {
                            let spread_ticks = ask.saturating_sub(bid);
                            Some(config.converter.ticks_to_price(spread_ticks))
                        }
                        _ => None,
                    };

                    snapshot.stats = Some(OrderbookStats {
                        vwap_bid: enriched
                            .vwap_bid
                            .map(|v| config.converter.ticks_to_price(v as u64).to_string()),
                        vwap_ask: enriched
                            .vwap_ask
                            .map(|v| config.converter.ticks_to_price(v as u64).to_string()),
                        spread: spread.map(|v| v.to_string()),
                        spread_bps: enriched.spread_bps.map(|v| format!("{:.2}", v)),
                        micro_price: enriched
                            .mid_price
                            .map(|v| config.converter.ticks_to_price(v as u64).to_string()),
                        imbalance: Some(enriched.order_book_imbalance),
                        bid_depth: Some(
                            config
                                .converter
                                .lots_to_size(enriched.bid_depth_total)
                                .to_string(),
                        ),
                        ask_depth: Some(
                            config
                                .converter
                                .lots_to_size(enriched.ask_depth_total)
                                .to_string(),
                        ),
                    });
                }
            }

            snapshots.push(snapshot);
        }

        snapshots
    }

    /// Get the number of managed orderbooks
    pub fn len(&self) -> usize {
        self.order_storage.len()
    }

    /// Check if there are no managed orderbooks
    pub fn is_empty(&self) -> bool {
        self.order_storage.is_empty()
    }

    /// Get the list of all managed market symbols
    pub async fn managed_symbols(&self) -> Vec<String> {
        let manager = self.manager.read().await;
        manager.symbols()
    }

    /// Get the number of books in the underlying manager
    pub async fn managed_book_count(&self) -> usize {
        let manager = self.manager.read().await;
        manager.book_count()
    }

    /// Start the trade event processor (Tokio async)
    ///
    /// This processor handles trade events from OrderBook-rs's internal TradeListener.
    pub async fn start_trade_processor(&self) -> tokio::task::JoinHandle<()> {
        let mut manager = self.manager.write().await;
        manager.start_trade_processor()
    }

    /// Check if a market has been initialized
    pub async fn has_market(&self, market_id: &str) -> bool {
        let manager = self.manager.read().await;
        manager.has_book(market_id)
    }

    /// Get an order by ID
    pub fn get_order(&self, order_id: Uuid) -> Option<Order> {
        let market_id = self.uuid_to_market.get(&order_id)?.clone();
        let market_orders = self.order_storage.get(&market_id)?;
        market_orders.get(&order_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::domain::{OrderStatus, OrderType};

    // BTC/USDC market config (from config.toml)
    const BTC_TICK_SIZE: u128 = 1_000_000;
    const BTC_LOT_SIZE: u128 = 10_000;
    const BTC_MIN_SIZE: u128 = 10_000;

    // ETH/USDC market config (example)
    const ETH_TICK_SIZE: u128 = 100_000;
    const ETH_LOT_SIZE: u128 = 100_000;
    const ETH_MIN_SIZE: u128 = 100_000;

    fn make_btc_market() -> Market {
        Market {
            id: "BTC/USDC".to_string(),
            base_ticker: "BTC".to_string(),
            quote_ticker: "USDC".to_string(),
            tick_size: BTC_TICK_SIZE,
            lot_size: BTC_LOT_SIZE,
            min_size: BTC_MIN_SIZE,
            maker_fee_bps: 5,
            taker_fee_bps: 10,
        }
    }

    fn make_eth_market() -> Market {
        Market {
            id: "ETH/USDC".to_string(),
            base_ticker: "ETH".to_string(),
            quote_ticker: "USDC".to_string(),
            tick_size: ETH_TICK_SIZE,
            lot_size: ETH_LOT_SIZE,
            min_size: ETH_MIN_SIZE,
            maker_fee_bps: 5,
            taker_fee_bps: 10,
        }
    }

    fn make_order(market_id: &str, user: &str, side: Side, price: u128, size: u128) -> Order {
        Order {
            id: Uuid::new_v4(),
            user_address: user.to_string(),
            market_id: market_id.to_string(),
            price,
            size,
            side,
            order_type: OrderType::Limit,
            status: OrderStatus::Pending,
            filled_size: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_book_manager_integration() {
        let manager = BookManagerAdapter::new();
        let btc_market = make_btc_market();

        // Add an order - this initializes the market and adds to BookManagerTokio
        let order = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        manager.add_order(&btc_market, &order).await;

        // Verify the book exists
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.managed_book_count().await, 1);

        let symbols = manager.managed_symbols().await;
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0], "BTC/USDC");

        // Verify market is initialized
        assert!(manager.has_market("BTC/USDC").await);
    }

    #[tokio::test]
    async fn test_cancel_order() {
        let manager = BookManagerAdapter::new();
        let btc_market = make_btc_market();

        let order = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order_id = order.id;

        manager.add_order(&btc_market, &order).await;

        // Cancel with correct user
        let result = manager.cancel_order(order_id, "user1").await;
        assert!(result.is_ok());

        // Try to cancel again (should fail)
        let result = manager.cancel_order(order_id, "user1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_order_wrong_user() {
        let manager = BookManagerAdapter::new();
        let btc_market = make_btc_market();

        let order = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order_id = order.id;

        manager.add_order(&btc_market, &order).await;

        // Cancel with wrong user
        let result = manager.cancel_order(order_id, "user2").await;
        assert!(result.is_err());

        // Order should still be in the book
        let snapshots = manager.snapshots();
        assert_eq!(snapshots[0].bids.len(), 1);
    }

    #[tokio::test]
    async fn test_cancel_all_orders() {
        let manager = BookManagerAdapter::new();
        let btc_market = make_btc_market();
        let eth_market = make_eth_market();

        // Add orders for user1 in both markets
        let order1 = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order2 = make_order(
            "ETH/USDC",
            "user1",
            Side::Sell,
            3_000_000_000,
            1_000_000_000,
        );

        // Add order for user2
        let order3 = make_order("BTC/USDC", "user2", Side::Buy, 49_000_000_000, 100_000_000);

        manager.add_order(&btc_market, &order1).await;
        manager.add_order(&eth_market, &order2).await;
        manager.add_order(&btc_market, &order3).await;

        // Cancel all user1 orders
        let cancelled = manager.cancel_all_orders("user1", None).await;
        assert_eq!(cancelled.len(), 2);

        // Only user2's order should remain
        let snapshots = manager.snapshots();
        let total_bids: usize = snapshots.iter().map(|s| s.bids.len()).sum();
        let total_asks: usize = snapshots.iter().map(|s| s.asks.len()).sum();
        assert_eq!(total_bids, 1);
        assert_eq!(total_asks, 0);
    }

    #[tokio::test]
    async fn test_cancel_all_orders_specific_market() {
        let manager = BookManagerAdapter::new();
        let btc_market = make_btc_market();
        let eth_market = make_eth_market();

        // Add orders for user1 in both markets
        let order1 = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order2 = make_order(
            "ETH/USDC",
            "user1",
            Side::Sell,
            3_000_000_000,
            1_000_000_000,
        );

        manager.add_order(&btc_market, &order1).await;
        manager.add_order(&eth_market, &order2).await;

        // Cancel user1 orders only in BTC/USDC
        let cancelled = manager.cancel_all_orders("user1", Some("BTC/USDC")).await;
        assert_eq!(cancelled.len(), 1);

        // ETH/USDC order should still exist
        let snapshots = manager.snapshots();
        let eth_snapshot = snapshots
            .iter()
            .find(|s| s.market_id == "ETH/USDC")
            .unwrap();
        assert_eq!(eth_snapshot.asks.len(), 1);
    }

    #[tokio::test]
    async fn test_order_matching() {
        let (event_tx, mut event_rx) = broadcast::channel(100);
        let manager = BookManagerAdapter::with_event_tx(event_tx);
        let btc_market = make_btc_market();

        // Add maker sell order
        let maker = make_order("BTC/USDC", "maker", Side::Sell, 50_000_000_000, 100_000_000);
        manager.add_order(&btc_market, &maker).await;

        // Create taker buy order
        let taker = make_order("BTC/USDC", "taker", Side::Buy, 50_000_000_000, 50_000_000);

        // Match the order
        let matches = manager.match_order(&btc_market, &taker).await;

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].size, 50_000_000);
        assert_eq!(matches[0].price, 50_000_000_000);

        // Verify trade event was emitted
        let event = event_rx.recv().await.unwrap();
        match event {
            EngineEvent::TradeExecuted { trade } => {
                assert_eq!(trade.size, 50_000_000);
                assert_eq!(trade.price, 50_000_000_000);
            }
            _ => panic!("Expected TradeExecuted event"),
        }
    }

    #[tokio::test]
    async fn test_get_order() {
        let manager = BookManagerAdapter::new();
        let btc_market = make_btc_market();

        let order = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order_id = order.id;

        manager.add_order(&btc_market, &order).await;

        // Should be able to retrieve the order
        let retrieved = manager.get_order(order_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, order_id);

        // Non-existent order should return None
        let not_found = manager.get_order(Uuid::new_v4());
        assert!(not_found.is_none());
    }

    #[cfg(test)]
    mod concurrent_tests {
        use super::*;
        use std::sync::Arc;
        use tokio::task::JoinSet;

        fn create_test_market(market_id: &str) -> Market {
            Market {
                id: market_id.to_string(),
                base_ticker: "BASE".to_string(),
                quote_ticker: "QUOTE".to_string(),
                tick_size: 1_000_000,
                lot_size: 10_000,
                min_size: 10_000,
                maker_fee_bps: 5,
                taker_fee_bps: 10,
            }
        }

        fn create_test_order(market_id: &str) -> Order {
            Order {
                id: Uuid::new_v4(),
                user_address: "test_user".to_string(),
                market_id: market_id.to_string(),
                price: 50_000_000_000,
                size: 1_000_000,
                side: Side::Buy,
                order_type: OrderType::Limit,
                status: OrderStatus::Pending,
                filled_size: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }

        #[tokio::test]
        async fn test_concurrent_operations_different_markets() {
            let manager = Arc::new(BookManagerAdapter::new());
            let mut tasks = JoinSet::new();

            // Spawn 100 concurrent tasks on different markets
            for i in 0..100 {
                let manager_clone = Arc::clone(&manager);
                tasks.spawn(async move {
                    let market = create_test_market(&format!("MARKET{}", i % 10));
                    let order = create_test_order(&market.id);

                    manager_clone.add_order(&market, &order).await;
                });
            }

            while let Some(task) = tasks.join_next().await {
                task.unwrap();
            }

            assert_eq!(manager.len(), 10);
        }

        #[tokio::test]
        async fn bench_cancel_order_with_index() {
            let manager = BookManagerAdapter::new();

            // Add 1000 orders across 100 markets
            let mut last_order_id = Uuid::new_v4();
            for market_idx in 0..100 {
                let market = create_test_market(&format!("MARKET{}", market_idx));
                for _order_idx in 0..10 {
                    let order = create_test_order(&market.id);
                    if market_idx == 0 && _order_idx == 0 {
                        last_order_id = order.id;
                    }
                    manager.add_order(&market, &order).await;
                }
            }

            // Test O(1) lookup performance
            use std::time::Instant;
            let start = Instant::now();
            let _ = manager.uuid_to_market.get(&last_order_id);
            let elapsed = start.elapsed();

            // Index lookup should be very fast (<100μs)
            assert!(elapsed < std::time::Duration::from_micros(100));

            // Test cancellation performance
            let start = Instant::now();
            let _ = manager.cancel_order(last_order_id, "test_user").await;
            let elapsed = start.elapsed();

            // Should be reasonably fast
            assert!(elapsed < std::time::Duration::from_millis(10));
        }
    }
}
