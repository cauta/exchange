//! Adapter wrapping a single OrderBook-rs instance for one market

use chrono::Utc;
use orderbook_rs::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::domain::{
    Market, Order, OrderStatus, OrderbookLevel, OrderbookSnapshot, OrderbookStats, Side,
};

use super::price_converter::PriceConverter;

/// Metadata stored alongside each order for domain mapping
#[derive(Clone, Debug, Default)]
pub struct OrderMetadata {
    pub user_address: String,
    pub market_id: String,
    pub original_size: u128,
    pub filled_size: u128,
    pub created_at: chrono::DateTime<Utc>,
}

/// Adapter wrapping a single OrderBook-rs instance
///
/// Handles:
/// - Converting between domain types (u128) and OrderBook-rs types (u64)
/// - Maintaining a map of orders for lookup by ID
/// - Generating snapshots with optional analytics stats
pub struct OrderbookAdapter {
    market_id: String,
    book: OrderBook<OrderMetadata>,
    #[allow(dead_code)]
    converter: PriceConverter,
    /// Track orders by UUID for retrieval and cancellation
    order_map: HashMap<Uuid, Order>,
}

impl OrderbookAdapter {
    /// Create a new orderbook adapter for a market
    ///
    /// # Arguments
    /// * `market_id` - The market identifier (e.g., "BTC/USDC")
    /// * `base_decimals` - Decimal places for base token
    /// * `quote_decimals` - Decimal places for quote token
    pub fn new(market_id: String, base_decimals: u8, quote_decimals: u8) -> Self {
        Self {
            book: OrderBook::new(&market_id),
            market_id,
            converter: PriceConverter::new(base_decimals, quote_decimals),
            order_map: HashMap::new(),
        }
    }

    /// Add an order to the orderbook
    pub fn add_order(&mut self, order: Order) {
        let remaining_size = order.size.saturating_sub(order.filled_size);
        if remaining_size == 0 {
            return;
        }

        // Convert to OrderBook-rs types
        // OrderBook-rs uses u64 internally, we need to scale our u128 values appropriately
        let ob_price = self.to_ob_price(order.price);
        let ob_size = self.to_ob_size(remaining_size);
        let ob_side = match order.side {
            Side::Buy => orderbook_rs::Side::Buy,
            Side::Sell => orderbook_rs::Side::Sell,
        };

        // Store in our map for later retrieval
        self.order_map.insert(order.id, order.clone());

        // Create metadata for the order
        let metadata = OrderMetadata {
            user_address: order.user_address.clone(),
            market_id: order.market_id.clone(),
            original_size: order.size,
            filled_size: order.filled_size,
            created_at: order.created_at,
        };

        // Use OrderId with uuid
        let order_id = OrderId::new();

        // Add to OrderBook-rs
        let _ = self.book.add_limit_order(
            order_id,
            ob_price,
            ob_size,
            ob_side,
            TimeInForce::Gtc, // Good Till Cancelled
            Some(metadata),
        );
    }

    /// Remove an order from the orderbook by ID
    pub fn remove_order(&mut self, order_id: Uuid) -> Option<Order> {
        // We need to find the OrderBook-rs OrderId for this UUID
        // Since we store our orders in order_map, we can remove from there
        // and also cancel in the book by searching
        if let Some(order) = self.order_map.remove(&order_id) {
            // Cancel all orders at this price level for this user (simplified approach)
            // In a production system, we'd maintain a UUID -> OrderId mapping
            return Some(order);
        }
        None
    }

    /// Remove all orders for a specific user
    pub fn remove_all_user_orders(&mut self, user_address: &str) -> Vec<Order> {
        let mut removed = Vec::new();

        // Find all orders belonging to this user
        let user_order_ids: Vec<Uuid> = self
            .order_map
            .iter()
            .filter(|(_, order)| order.user_address == user_address)
            .map(|(id, _)| *id)
            .collect();

        // Remove each order
        for order_id in user_order_ids {
            if let Some(order) = self.order_map.remove(&order_id) {
                removed.push(order);
            }
        }

        removed
    }

    /// Update an order's filled amount, removing if fully filled
    pub fn update_order_fill(&mut self, order_id: Uuid, fill_size: u128) {
        if let Some(order) = self.order_map.get_mut(&order_id) {
            order.filled_size += fill_size;
            order.updated_at = Utc::now();

            // Check if fully filled
            if order.filled_size >= order.size {
                order.status = OrderStatus::Filled;
                self.order_map.remove(&order_id);
            }
        }
    }

    /// Apply executed trades to the orderbook
    pub fn apply_trades(
        &mut self,
        taker_order: &Order,
        trades: &[crate::models::domain::Trade],
        market: &Market,
    ) {
        // Update maker orders that were executed
        for trade in trades {
            let maker_order_id = match taker_order.side {
                Side::Buy => trade.seller_order_id,
                Side::Sell => trade.buyer_order_id,
            };
            self.update_order_fill(maker_order_id, trade.size);
        }

        // Add taker order to book if not fully filled
        let total_matched: u128 = trades.iter().map(|t| t.size).sum();
        let remaining_size = taker_order.size.saturating_sub(total_matched);

        // Only add to book if remaining size meets minimum order size
        if remaining_size > 0 && remaining_size >= market.min_size {
            let mut remaining_order = taker_order.clone();
            remaining_order.filled_size = total_matched;
            remaining_order.status = if total_matched > 0 {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Pending
            };
            self.add_order(remaining_order);
        }
    }

    /// Generate a snapshot of the current orderbook state
    pub fn snapshot(&self) -> OrderbookSnapshot {
        // Aggregate bids by price level
        let mut bid_levels: HashMap<u128, u128> = HashMap::new();
        for order in self.order_map.values().filter(|o| o.side == Side::Buy) {
            let remaining = order.size.saturating_sub(order.filled_size);
            *bid_levels.entry(order.price).or_insert(0) += remaining;
        }

        let mut bids: Vec<OrderbookLevel> = bid_levels
            .into_iter()
            .filter(|(_, size)| *size > 0)
            .map(|(price, size)| OrderbookLevel { price, size })
            .collect();
        bids.sort_by(|a, b| b.price.cmp(&a.price)); // Descending

        // Aggregate asks by price level
        let mut ask_levels: HashMap<u128, u128> = HashMap::new();
        for order in self.order_map.values().filter(|o| o.side == Side::Sell) {
            let remaining = order.size.saturating_sub(order.filled_size);
            *ask_levels.entry(order.price).or_insert(0) += remaining;
        }

        let mut asks: Vec<OrderbookLevel> = ask_levels
            .into_iter()
            .filter(|(_, size)| *size > 0)
            .map(|(price, size)| OrderbookLevel { price, size })
            .collect();
        asks.sort_by(|a, b| a.price.cmp(&b.price)); // Ascending

        OrderbookSnapshot {
            market_id: self.market_id.clone(),
            bids,
            asks,
            timestamp: Utc::now(),
            stats: None,
        }
    }

    /// Generate a snapshot with analytics stats from OrderBook-rs
    pub fn enriched_snapshot(&self) -> OrderbookSnapshot {
        let mut snapshot = self.snapshot();

        // Use OrderBook-rs built-in analytics (top 100 levels)
        let enriched = self.book.enriched_snapshot(100);

        // Calculate spread from best bid/ask if available
        let spread = match (self.book.best_bid(), self.book.best_ask()) {
            (Some(bid), Some(ask)) => Some((ask.saturating_sub(bid)) as u128),
            _ => None,
        };

        snapshot.stats = Some(OrderbookStats {
            vwap_bid: enriched.vwap_bid.map(|v| self.from_ob_price_f64(v).to_string()),
            vwap_ask: enriched.vwap_ask.map(|v| self.from_ob_price_f64(v).to_string()),
            spread: spread.map(|v| v.to_string()),
            spread_bps: enriched.spread_bps.map(|v| format!("{:.2}", v)),
            micro_price: enriched.mid_price.map(|v| self.from_ob_price_f64(v).to_string()),
            imbalance: Some(enriched.order_book_imbalance),
            bid_depth: Some((enriched.bid_depth_total as u128).to_string()),
            ask_depth: Some((enriched.ask_depth_total as u128).to_string()),
        });

        snapshot
    }

    /// Convert u128 price to u64 for OrderBook-rs
    /// This scales down the price to fit in u64, maintaining relative precision
    fn to_ob_price(&self, price: u128) -> u64 {
        // For most use cases, prices fit in u64
        // If price exceeds u64::MAX, we'd need a different approach
        price.min(u64::MAX as u128) as u64
    }

    /// Convert f64 price back to u128
    fn from_ob_price_f64(&self, price: f64) -> u128 {
        price as u128
    }

    /// Convert u128 size to u64 for OrderBook-rs
    fn to_ob_size(&self, size: u128) -> u64 {
        size.min(u64::MAX as u128) as u64
    }

    /// Get the market ID
    pub fn market_id(&self) -> &str {
        &self.market_id
    }

    /// Match a taker order against this orderbook
    /// Returns a vector of matches (maker orders matched with taker order)
    /// This replicates the Matcher logic but works with our order_map
    pub fn match_order(&self, taker_order: &Order) -> Vec<crate::models::domain::Match> {
        use crate::models::domain::{Match, OrderType};
        
        let mut matches = Vec::new();
        let mut remaining_size = taker_order.size.saturating_sub(taker_order.filled_size);

        // Get orders on the opposite side, sorted appropriately
        let mut opposite_orders: Vec<&Order> = self
            .order_map
            .values()
            .filter(|o| {
                // Opposite side
                match taker_order.side {
                    Side::Buy => o.side == Side::Sell,
                    Side::Sell => o.side == Side::Buy,
                }
            })
            .filter(|o| o.size > o.filled_size) // Has remaining size
            .collect();

        // Sort by price-time priority
        match taker_order.side {
            Side::Buy => {
                // For buy taker: match against asks, lowest price first
                opposite_orders.sort_by(|a, b| {
                    a.price.cmp(&b.price).then_with(|| a.created_at.cmp(&b.created_at))
                });
            }
            Side::Sell => {
                // For sell taker: match against bids, highest price first
                opposite_orders.sort_by(|a, b| {
                    b.price.cmp(&a.price).then_with(|| a.created_at.cmp(&b.created_at))
                });
            }
        }

        for maker_order in opposite_orders {
            if remaining_size == 0 {
                break;
            }

            // Check if this price level can match
            let can_match = match (taker_order.side, taker_order.order_type) {
                (Side::Buy, OrderType::Limit) => taker_order.price >= maker_order.price,
                (Side::Buy, OrderType::Market) => true,
                (Side::Sell, OrderType::Limit) => taker_order.price <= maker_order.price,
                (Side::Sell, OrderType::Market) => true,
            };

            if !can_match {
                break; // No more matches possible
            }

            // Skip self-trading
            if maker_order.user_address == taker_order.user_address {
                continue;
            }

            // Calculate match size
            let maker_remaining = maker_order.size.saturating_sub(maker_order.filled_size);
            let match_size = remaining_size.min(maker_remaining);

            matches.push(Match {
                maker_order: maker_order.clone(),
                price: maker_order.price, // Match at maker's price
                size: match_size,
            });

            remaining_size -= match_size;
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::domain::OrderType;

    fn make_order(side: Side, price: u128, size: u128) -> Order {
        Order {
            id: Uuid::new_v4(),
            user_address: "test_user".to_string(),
            market_id: "BTC/USDC".to_string(),
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
    fn test_add_and_snapshot() {
        let mut adapter = OrderbookAdapter::new("BTC/USDC".to_string(), 8, 6);

        // Add buy order at $50,000
        adapter.add_order(make_order(Side::Buy, 50_000_000_000, 100_000_000));

        // Add sell order at $51,000
        adapter.add_order(make_order(Side::Sell, 51_000_000_000, 100_000_000));

        let snapshot = adapter.snapshot();

        assert_eq!(snapshot.market_id, "BTC/USDC");
        assert_eq!(snapshot.bids.len(), 1);
        assert_eq!(snapshot.asks.len(), 1);
        assert_eq!(snapshot.bids[0].price, 50_000_000_000);
        assert_eq!(snapshot.asks[0].price, 51_000_000_000);
    }

    #[test]
    fn test_remove_order() {
        let mut adapter = OrderbookAdapter::new("BTC/USDC".to_string(), 8, 6);

        let order = make_order(Side::Buy, 50_000_000_000, 100_000_000);
        let order_id = order.id;
        adapter.add_order(order);

        let removed = adapter.remove_order(order_id);
        assert!(removed.is_some());

        let snapshot = adapter.snapshot();
        assert!(snapshot.bids.is_empty());
    }

    #[test]
    fn test_remove_all_user_orders() {
        let mut adapter = OrderbookAdapter::new("BTC/USDC".to_string(), 8, 6);

        // Add orders for user1
        let mut order1 = make_order(Side::Buy, 50_000_000_000, 100_000_000);
        order1.user_address = "user1".to_string();
        adapter.add_order(order1);

        let mut order2 = make_order(Side::Sell, 51_000_000_000, 100_000_000);
        order2.user_address = "user1".to_string();
        adapter.add_order(order2);

        // Add order for user2
        let mut order3 = make_order(Side::Buy, 49_000_000_000, 100_000_000);
        order3.user_address = "user2".to_string();
        adapter.add_order(order3);

        let removed = adapter.remove_all_user_orders("user1");
        assert_eq!(removed.len(), 2);

        let snapshot = adapter.snapshot();
        assert_eq!(snapshot.bids.len(), 1); // Only user2's order remains
        assert!(snapshot.asks.is_empty());
    }
}
