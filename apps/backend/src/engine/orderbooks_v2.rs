//! V2 Orderbooks implementation using OrderBook-rs
//!
//! This module provides the same public interface as `orderbook.rs` but uses
//! OrderBook-rs under the hood for improved performance via lock-free data structures.

use crate::errors::Result;
use crate::models::domain::{Market, Order, OrderbookSnapshot};
use uuid::Uuid;

use super::adapter::{BookManagerAdapter, OrderbookAdapter};

/// V2 Orderbooks implementation using OrderBook-rs
///
/// This maintains the same public interface as the original `Orderbooks` struct
/// to allow easy switching via feature flag.
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

    /// Get or create a mutable reference to an orderbook for a market
    ///
    /// Uses the market's tick_size and lot_size for proper price/size scaling.
    pub fn get_or_create(&self, market: &Market) -> dashmap::mapref::one::RefMut<'_, String, OrderbookAdapter> {
        self.manager.get_or_create(market)
    }

    /// Get an existing orderbook by market_id (without creating)
    pub fn get(&self, market_id: &str) -> Option<dashmap::mapref::one::RefMut<'_, String, OrderbookAdapter>> {
        self.manager.get(market_id)
    }

    /// Cancel an order across all markets
    ///
    /// Returns the cancelled order if found and ownership is verified.
    pub fn cancel_order(&self, order_id: Uuid, user_address: &str) -> Result<Order> {
        self.manager.cancel_order(order_id, user_address)
    }

    /// Cancel all orders for a user, optionally filtered by market
    ///
    /// Returns a vector of all cancelled orders.
    pub fn cancel_all_orders(&self, user_address: &str, market_id: Option<&str>) -> Vec<Order> {
        self.manager.cancel_all_orders(user_address, market_id)
    }

    /// Generate snapshots for all markets (without stats)
    pub fn snapshots(&self) -> Vec<OrderbookSnapshot> {
        self.manager.snapshots()
    }

    /// Generate snapshots for all markets with analytics stats
    ///
    /// This provides additional market analytics from OrderBook-rs:
    /// - VWAP (bid/ask)
    /// - Spread (absolute and bps)
    /// - Micro price
    /// - Order book imbalance
    /// - Total depth (bid/ask)
    pub fn enriched_snapshots(&self) -> Vec<OrderbookSnapshot> {
        self.manager.enriched_snapshots()
    }
}

/// Wrapper around OrderbookAdapter to provide compatibility with the original Orderbook interface
///
/// This allows the matching engine to work with both V1 (original) and V2 (OrderBook-rs) orderbooks
/// with minimal code changes.
impl OrderbookAdapter {
    /// Apply executed trades to the orderbook (compatibility method)
    ///
    /// This delegates to the adapter's apply_trades method.
    pub fn apply_trades_compat(
        &mut self,
        taker_order: &Order,
        trades: &[crate::models::domain::Trade],
        market: &Market,
    ) {
        self.apply_trades(taker_order, trades, market);
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

    fn make_order(market_id: &str, side: Side, price: u128, size: u128) -> Order {
        Order {
            id: Uuid::new_v4(),
            user_address: "test_user".to_string(),
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

    #[test]
    fn test_orderbooks_v2_basic() {
        let orderbooks = OrderbooksV2::new();
        let btc_market = make_btc_market();

        // Add orders
        let mut book = orderbooks.get_or_create(&btc_market);
        book.add_order(make_order(
            "BTC/USDC",
            Side::Buy,
            50_000_000_000,
            100_000_000,
        ));
        book.add_order(make_order(
            "BTC/USDC",
            Side::Sell,
            51_000_000_000,
            100_000_000,
        ));

        // Get snapshots
        let snapshots = orderbooks.snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].bids.len(), 1);
        assert_eq!(snapshots[0].asks.len(), 1);
    }

    #[test]
    fn test_orderbooks_v2_enriched_snapshots() {
        let orderbooks = OrderbooksV2::new();
        let btc_market = make_btc_market();

        // Add orders
        let mut book = orderbooks.get_or_create(&btc_market);
        book.add_order(make_order(
            "BTC/USDC",
            Side::Buy,
            50_000_000_000,
            100_000_000,
        ));
        book.add_order(make_order(
            "BTC/USDC",
            Side::Sell,
            51_000_000_000,
            100_000_000,
        ));

        // Get enriched snapshots
        let snapshots = orderbooks.enriched_snapshots();
        assert_eq!(snapshots.len(), 1);

        // Stats should be present
        let stats = snapshots[0].stats.as_ref();
        assert!(stats.is_some());
    }
}
