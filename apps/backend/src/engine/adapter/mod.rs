//! Adapter layer for OrderBook-rs integration
//!
//! This module provides adapters that wrap OrderBook-rs types while maintaining
//! compatibility with our domain types (u128 prices, Uuid order IDs, etc.)

pub mod book_manager_adapter;
pub mod orderbook_adapter;
pub mod price_converter;

pub use book_manager_adapter::BookManagerAdapter;
pub use orderbook_adapter::OrderbookAdapter;
pub use price_converter::PriceConverter;
