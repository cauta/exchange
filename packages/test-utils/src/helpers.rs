use crate::db::TestDb;
use backend::models::domain::{Market, Token, Trade, User};

// ============================================================================
// Database Helpers - Direct DB Access for Backend Tests
// ============================================================================

/// Create a test user
pub async fn create_user(test_db: &TestDb, address: &str) -> anyhow::Result<User> {
    test_db
        .db
        .create_user(address.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create test user: {}", e))
}

/// Create a test token
pub async fn create_token(
    test_db: &TestDb,
    ticker: &str,
    decimals: u8,
    name: &str,
) -> anyhow::Result<Token> {
    test_db
        .db
        .create_token(ticker.to_string(), decimals, name.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create test token: {}", e))
}

/// Create a test market
///
/// Requires that both base and quote tokens already exist.
pub async fn create_market(
    test_db: &TestDb,
    base_ticker: &str,
    quote_ticker: &str,
) -> anyhow::Result<Market> {
    test_db
        .db
        .create_market(
            base_ticker.to_string(),
            quote_ticker.to_string(),
            1000,    // tick_size
            1000000, // lot_size
            1000000, // min_size
            10,      // maker_fee_bps
            20,      // taker_fee_bps
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create test market: {}", e))
}

/// Create a test market with tokens
///
/// Creates the tokens first (with realistic decimals), then the market.
/// Base tokens use 8 decimals, quote tokens use 6 decimals.
pub async fn create_market_with_tokens(
    test_db: &TestDb,
    base_ticker: &str,
    quote_ticker: &str,
) -> anyhow::Result<Market> {
    // Create tokens with realistic decimals
    let base_decimals = 8;
    let quote_decimals = 6;

    create_token(
        test_db,
        base_ticker,
        base_decimals,
        &format!("{} Token", base_ticker),
    )
    .await?;

    create_token(
        test_db,
        quote_ticker,
        quote_decimals,
        &format!("{} Token", quote_ticker),
    )
    .await?;

    create_market(test_db, base_ticker, quote_ticker).await
}

/// Create test candle by inserting trades
///
/// Generates candles via materialized views (the real production flow).
/// Inserts 4 trades to create an OHLCV pattern.
pub async fn create_candle(
    test_db: &TestDb,
    market_id: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
    ohlcv: (u128, u128, u128, u128, u128), // (open, high, low, close, volume)
) -> anyhow::Result<()> {
    let (open, high, low, close, volume) = ohlcv;

    let trades = [
        (open, timestamp.timestamp() as u32),      // Open
        (high, timestamp.timestamp() as u32 + 1),  // High
        (low, timestamp.timestamp() as u32 + 2),   // Low
        (close, timestamp.timestamp() as u32 + 3), // Close
    ];

    let trade_size = volume / 4; // Divide volume across 4 trades

    for (i, (price, ts)) in trades.iter().enumerate() {
        let trade = Trade {
            id: uuid::Uuid::new_v4(),
            market_id: market_id.to_string(),
            buyer_address: format!("test_buyer_{}", i),
            seller_address: format!("test_seller_{}", i),
            buyer_order_id: uuid::Uuid::new_v4(),
            seller_order_id: uuid::Uuid::new_v4(),
            price: *price,
            size: trade_size,
            side: backend::models::domain::Side::Buy,
            timestamp: chrono::DateTime::from_timestamp(*ts as i64, 0)
                .unwrap_or(chrono::DateTime::UNIX_EPOCH),
        };

        test_db
            .db
            .insert_trade_to_clickhouse(&trade)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to insert trade for candle: {}", e))?;
    }

    // Give ClickHouse time to process the materialized views
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    Ok(())
}
