//! Adapter for managing multiple OrderBook-rs instances across markets

use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::{ExchangeError, Result};
use crate::models::domain::{Order, OrderbookSnapshot};

use super::orderbook_adapter::OrderbookAdapter;

/// Configuration for a market's decimal precision
#[derive(Debug, Clone, Copy)]
pub struct MarketDecimals {
    pub base_decimals: u8,
    pub quote_decimals: u8,
}

impl Default for MarketDecimals {
    fn default() -> Self {
        Self {
            base_decimals: 18,
            quote_decimals: 6,
        }
    }
}

/// Manages multiple OrderbookAdapter instances across markets
///
/// This is the V2 equivalent of the `Orderbooks` struct, using OrderBook-rs
/// under the hood for improved performance.
pub struct BookManagerAdapter {
    /// Market decimal configurations
    market_decimals: HashMap<String, MarketDecimals>,
    /// Individual orderbook adapters per market
    books: HashMap<String, OrderbookAdapter>,
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
            market_decimals: HashMap::new(),
            books: HashMap::new(),
        }
    }

    /// Configure decimal precision for a market
    ///
    /// This should be called before any orders are added to the market
    /// to ensure correct price/size conversion.
    pub fn configure_market(&mut self, market_id: &str, base_decimals: u8, quote_decimals: u8) {
        self.market_decimals.insert(
            market_id.to_string(),
            MarketDecimals {
                base_decimals,
                quote_decimals,
            },
        );
    }

    /// Get or create an orderbook adapter for a market
    ///
    /// If the market hasn't been configured, uses default decimals (18, 6).
    pub fn get_or_create(&mut self, market_id: &str) -> &mut OrderbookAdapter {
        if !self.books.contains_key(market_id) {
            let decimals = self
                .market_decimals
                .get(market_id)
                .copied()
                .unwrap_or_default();

            self.books.insert(
                market_id.to_string(),
                OrderbookAdapter::new(
                    market_id.to_string(),
                    decimals.base_decimals,
                    decimals.quote_decimals,
                ),
            );
        }
        self.books.get_mut(market_id).unwrap()
    }

    /// Cancel an order across all markets
    ///
    /// Returns the cancelled order if found and ownership is verified.
    pub fn cancel_order(&mut self, order_id: Uuid, user_address: &str) -> Result<Order> {
        // Search all markets for the order
        for book in self.books.values_mut() {
            if let Some(order) = book.remove_order(order_id) {
                // Verify ownership
                if order.user_address != user_address {
                    // Put the order back since ownership check failed
                    book.add_order(order);
                    return Err(ExchangeError::OrderNotFound);
                }
                return Ok(order);
            }
        }

        Err(ExchangeError::OrderNotFound)
    }

    /// Cancel all orders for a user, optionally filtered by market
    ///
    /// Returns a vector of all cancelled orders.
    pub fn cancel_all_orders(&mut self, user_address: &str, market_id: Option<&str>) -> Vec<Order> {
        let mut cancelled_orders = Vec::new();

        if let Some(market) = market_id {
            // Cancel orders in specific market only
            if let Some(book) = self.books.get_mut(market) {
                cancelled_orders.extend(book.remove_all_user_orders(user_address));
            }
        } else {
            // Cancel orders across all markets
            for book in self.books.values_mut() {
                cancelled_orders.extend(book.remove_all_user_orders(user_address));
            }
        }

        cancelled_orders
    }

    /// Generate snapshots for all markets (without stats)
    pub fn snapshots(&self) -> Vec<OrderbookSnapshot> {
        self.books.values().map(|book| book.snapshot()).collect()
    }

    /// Generate snapshots for all markets with analytics stats
    pub fn enriched_snapshots(&self) -> Vec<OrderbookSnapshot> {
        self.books
            .values()
            .map(|book| book.enriched_snapshot())
            .collect()
    }

    /// Get the number of managed orderbooks
    pub fn len(&self) -> usize {
        self.books.len()
    }

    /// Check if there are no managed orderbooks
    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::domain::{OrderStatus, OrderType, Side};
    use chrono::Utc;

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

    #[test]
    fn test_get_or_create() {
        let mut manager = BookManagerAdapter::new();

        // Configure market
        manager.configure_market("BTC/USDC", 8, 6);

        // Get orderbook (creates it)
        let book = manager.get_or_create("BTC/USDC");
        assert_eq!(book.market_id(), "BTC/USDC");

        // Get same orderbook (returns existing)
        let _book = manager.get_or_create("BTC/USDC");
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_cancel_order() {
        let mut manager = BookManagerAdapter::new();
        manager.configure_market("BTC/USDC", 8, 6);

        let order = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order_id = order.id;

        manager.get_or_create("BTC/USDC").add_order(order);

        // Cancel with correct user
        let result = manager.cancel_order(order_id, "user1");
        assert!(result.is_ok());

        // Try to cancel again (should fail)
        let result = manager.cancel_order(order_id, "user1");
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_order_wrong_user() {
        let mut manager = BookManagerAdapter::new();
        manager.configure_market("BTC/USDC", 8, 6);

        let order = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order_id = order.id;

        manager.get_or_create("BTC/USDC").add_order(order);

        // Cancel with wrong user
        let result = manager.cancel_order(order_id, "user2");
        assert!(result.is_err());

        // Order should still be in the book
        let snapshots = manager.snapshots();
        assert_eq!(snapshots[0].bids.len(), 1);
    }

    #[test]
    fn test_cancel_all_orders() {
        let mut manager = BookManagerAdapter::new();
        manager.configure_market("BTC/USDC", 8, 6);
        manager.configure_market("ETH/USDC", 18, 6);

        // Add orders for user1 in both markets
        let order1 = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order2 = make_order("ETH/USDC", "user1", Side::Sell, 3_000_000_000, 1_000_000_000);

        // Add order for user2
        let order3 = make_order("BTC/USDC", "user2", Side::Buy, 49_000_000_000, 100_000_000);

        manager.get_or_create("BTC/USDC").add_order(order1);
        manager.get_or_create("ETH/USDC").add_order(order2);
        manager.get_or_create("BTC/USDC").add_order(order3);

        // Cancel all user1 orders
        let cancelled = manager.cancel_all_orders("user1", None);
        assert_eq!(cancelled.len(), 2);

        // Only user2's order should remain
        let snapshots = manager.snapshots();
        let total_bids: usize = snapshots.iter().map(|s| s.bids.len()).sum();
        let total_asks: usize = snapshots.iter().map(|s| s.asks.len()).sum();
        assert_eq!(total_bids, 1);
        assert_eq!(total_asks, 0);
    }

    #[test]
    fn test_cancel_all_orders_specific_market() {
        let mut manager = BookManagerAdapter::new();
        manager.configure_market("BTC/USDC", 8, 6);
        manager.configure_market("ETH/USDC", 18, 6);

        // Add orders for user1 in both markets
        let order1 = make_order("BTC/USDC", "user1", Side::Buy, 50_000_000_000, 100_000_000);
        let order2 = make_order("ETH/USDC", "user1", Side::Sell, 3_000_000_000, 1_000_000_000);

        manager.get_or_create("BTC/USDC").add_order(order1);
        manager.get_or_create("ETH/USDC").add_order(order2);

        // Cancel user1 orders only in BTC/USDC
        let cancelled = manager.cancel_all_orders("user1", Some("BTC/USDC"));
        assert_eq!(cancelled.len(), 1);

        // ETH/USDC order should still exist
        let snapshots = manager.snapshots();
        let eth_snapshot = snapshots.iter().find(|s| s.market_id == "ETH/USDC").unwrap();
        assert_eq!(eth_snapshot.asks.len(), 1);
    }
}
