//! V2 Orderbooks implementation using OrderBook-rs
//!
//! This module provides an async interface backed by BookManagerTokio from orderbook-rs.
//! Uses lock-free data structures for improved concurrency and performance.

use crate::errors::Result;
use crate::models::domain::{EngineEvent, Market, Match, Order, OrderbookSnapshot, Trade};
use tokio::sync::broadcast;
use uuid::Uuid;

use super::adapter::BookManagerAdapter;

/// V2 Orderbooks implementation using OrderBook-rs
///
/// This uses BookManagerTokio from orderbook-rs for:
/// - Lock-free orderbook operations via crossbeam-skiplist
/// - Async-compatible trade event processing
/// - Centralized trade event routing
pub struct OrderbooksV2 {
    manager: BookManagerAdapter,
}

impl Default for OrderbooksV2 {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderbooksV2 {
    /// Create a new OrderbooksV2 instance
    pub fn new() -> Self {
        Self {
            manager: BookManagerAdapter::new(),
        }
    }

    /// Create a new OrderbooksV2 instance with event broadcasting
    pub fn with_event_tx(event_tx: broadcast::Sender<EngineEvent>) -> Self {
        Self {
            manager: BookManagerAdapter::with_event_tx(event_tx),
        }
    }

    /// Initialize a market's orderbook
    pub async fn init_market(&self, market: &Market) {
        self.manager.init_market(market).await;
    }

    /// Add an order to the orderbook
    pub async fn add_order(&self, market: &Market, order: &Order) {
        self.manager.add_order(market, order).await;
    }

    /// Match an order against the orderbook
    ///
    /// Returns matches and optionally emits trade events if configured.
    pub async fn match_order(&self, market: &Market, order: &Order) -> Vec<Match> {
        self.manager.match_order(market, order).await
    }

    /// Apply executed trades to the orderbook
    ///
    /// Updates maker order fill amounts and adds remaining taker order if applicable.
    pub async fn apply_trades(&self, taker_order: &Order, trades: &[Trade], market: &Market) {
        self.manager.apply_trades(taker_order, trades, market).await;
    }

    /// Cancel an order
    ///
    /// Returns the cancelled order if found and ownership is verified.
    pub async fn cancel_order(&self, order_id: Uuid, user_address: &str) -> Result<Order> {
        self.manager.cancel_order(order_id, user_address).await
    }

    /// Cancel all orders for a user, optionally filtered by market
    pub async fn cancel_all_orders(
        &self,
        user_address: &str,
        market_id: Option<&str>,
    ) -> Vec<Order> {
        self.manager
            .cancel_all_orders(user_address, market_id)
            .await
    }

    /// Generate snapshots for all markets
    pub fn snapshots(&self) -> Vec<OrderbookSnapshot> {
        self.manager.snapshots()
    }

    /// Generate snapshots with analytics stats from OrderBook-rs
    pub async fn enriched_snapshots(&self) -> Vec<OrderbookSnapshot> {
        self.manager.enriched_snapshots().await
    }

    /// Update an order's filled amount
    pub async fn update_order_fill(&self, order_id: Uuid, fill_size: u128) {
        self.manager.update_order_fill(order_id, fill_size).await;
    }

    /// Get the number of managed orderbooks
    pub fn len(&self) -> usize {
        self.manager.len()
    }

    /// Check if there are no managed orderbooks
    pub fn is_empty(&self) -> bool {
        self.manager.is_empty()
    }

    /// Start the trade event processor
    pub async fn start_trade_processor(&self) -> tokio::task::JoinHandle<()> {
        self.manager.start_trade_processor().await
    }

    /// Get an order by ID
    pub fn get_order(&self, order_id: Uuid) -> Option<Order> {
        self.manager.get_order(order_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::domain::{OrderStatus, OrderType, Side};
    use chrono::Utc;

    // BTC/USDC market config (from config.toml)
    const BTC_TICK_SIZE: u128 = 1_000_000;
    const BTC_LOT_SIZE: u128 = 10_000;
    const BTC_MIN_SIZE: u128 = 10_000;

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
    async fn test_orderbooks_v2_basic() {
        let orderbooks = OrderbooksV2::new();
        let btc_market = make_btc_market();

        // Add orders
        orderbooks
            .add_order(
                &btc_market,
                &make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000),
            )
            .await;
        orderbooks
            .add_order(
                &btc_market,
                &make_order("BTC/USDC", "user1", Side::Sell, 51_000_000_000, 100_000_000),
            )
            .await;

        // Get snapshots
        let snapshots = orderbooks.snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].bids.len(), 1);
        assert_eq!(snapshots[0].asks.len(), 1);
    }

    #[tokio::test]
    async fn test_orderbooks_v2_enriched_snapshots() {
        let orderbooks = OrderbooksV2::new();
        let btc_market = make_btc_market();

        // Add orders
        orderbooks
            .add_order(
                &btc_market,
                &make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000),
            )
            .await;
        orderbooks
            .add_order(
                &btc_market,
                &make_order("BTC/USDC", "user1", Side::Sell, 51_000_000_000, 100_000_000),
            )
            .await;

        // Get enriched snapshots
        let snapshots = orderbooks.enriched_snapshots().await;
        assert_eq!(snapshots.len(), 1);

        // Stats should be present
        let stats = snapshots[0].stats.as_ref();
        assert!(stats.is_some());
    }

    #[tokio::test]
    async fn test_orderbooks_v2_cancel_order() {
        let orderbooks = OrderbooksV2::new();
        let btc_market = make_btc_market();

        let order = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order_id = order.id;

        orderbooks.add_order(&btc_market, &order).await;

        // Cancel the order
        let result = orderbooks.cancel_order(order_id, "user1").await;
        assert!(result.is_ok());

        // Verify order is removed
        let snapshots = orderbooks.snapshots();
        assert!(snapshots[0].bids.is_empty());
    }

    #[tokio::test]
    async fn test_orderbooks_v2_match_order() {
        let orderbooks = OrderbooksV2::new();
        let btc_market = make_btc_market();

        // Add maker sell order
        let maker = make_order("BTC/USDC", "maker", Side::Sell, 50_000_000_000, 100_000_000);
        orderbooks.add_order(&btc_market, &maker).await;

        // Match with taker buy order
        let taker = make_order("BTC/USDC", "taker", Side::Buy, 50_000_000_000, 50_000_000);
        let matches = orderbooks.match_order(&btc_market, &taker).await;

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].size, 50_000_000);
        assert_eq!(matches[0].price, 50_000_000_000);
    }
}
