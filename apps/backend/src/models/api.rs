use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::domain::{Order, Trade};

// ============================================================================
// REST API TYPES
// ============================================================================

#[derive(Serialize, ToSchema)]
pub struct ApiResponse {
    pub message: String,
    pub timestamp: u64,
}

/// Response after successfully placing an order
#[derive(Debug, Clone, Serialize)]
pub struct OrderPlaced {
    pub order: Order,
    pub trades: Vec<Trade>,
}

/// Response after successfully cancelling an order
#[derive(Debug, Clone, Serialize)]
pub struct OrderCancelled {
    pub order_id: Uuid,
}

// ============================================================================
// WEBSOCKET MESSAGE TYPES (Client → Server)
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Subscribe {
        channel: String, // "trades", "orderbook", "candles", "user"
        market_id: Option<String>,
        user_address: Option<String>,
    },
    Unsubscribe {
        channel: String,
        market_id: Option<String>,
    },
    Ping,
}

// ============================================================================
// WEBSOCKET MESSAGE TYPES (Server → Client)
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Subscribed {
        channel: String,
        market_id: Option<String>,
    },
    Unsubscribed {
        channel: String,
        market_id: Option<String>,
    },
    Trade {
        market_id: String,
        price: String,
        size: String,
        side: String,
        timestamp: i64,
    },
    OrderbookUpdate {
        market_id: String,
        bids: Vec<PriceLevel>,
        asks: Vec<PriceLevel>,
    },
    OrderUpdate {
        order_id: String,
        status: String,
        filled_size: String,
    },
    BalanceUpdate {
        token_ticker: String,
        available: String,
        locked: String,
    },
    Candle {
        market_id: String,
        timestamp: i64,
        open: String,
        high: String,
        low: String,
        close: String,
        volume: String,
    },
    Error {
        message: String,
    },
    Pong,
}

#[derive(Debug, Serialize)]
pub struct PriceLevel {
    pub price: String,
    pub size: String,
}

// ============================================================================
// SUBSCRIPTION TYPES
// ============================================================================

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Subscription {
    Trades { market_id: String },
    Orderbook { market_id: String },
    Candles { market_id: String },
    User { user_address: String },
}

impl Subscription {
    /// Create a subscription from a channel name and optional identifiers
    pub fn from_channel(
        channel: &str,
        market_id: Option<String>,
        user_address: Option<String>,
    ) -> Option<Self> {
        match channel {
            "trades" => market_id.map(|id| Subscription::Trades { market_id: id }),
            "orderbook" => market_id.map(|id| Subscription::Orderbook { market_id: id }),
            "candles" => market_id.map(|id| Subscription::Candles { market_id: id }),
            "user" => user_address.map(|addr| Subscription::User { user_address: addr }),
            _ => None,
        }
    }
}
