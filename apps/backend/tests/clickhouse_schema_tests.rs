/// Tests to verify ClickHouse schema matches Rust structs
/// These tests catch schema mismatches that cause runtime panics
use backend::models::db::ClickHouseTradeRow;
use exchange_test_utils::TestContainers;

#[tokio::test]
async fn test_clickhouse_trades_schema_matches_struct() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    // Try to insert a dummy trade row
    let trade = ClickHouseTradeRow {
        id: "test-id-123".to_string(),
        market_id: "BTC/USDC".to_string(),
        buyer_address: "buyer".to_string(),
        seller_address: "seller".to_string(),
        buyer_order_id: "buyer-order-id".to_string(),
        seller_order_id: "seller-order-id".to_string(),
        price: 95000000000,
        size: 1000000,
        side: "buy".to_string(),
        timestamp: 1234567890,
    };

    // This will panic if schema doesn't match struct
    let result = db
        .clickhouse
        .insert::<ClickHouseTradeRow>("trades")
        .await
        .unwrap()
        .write(&trade)
        .await;

    assert!(
        result.is_ok(),
        "Failed to insert trade - schema mismatch! Error: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_candles_table_uses_aggregating_merge_tree() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    // Verify the candles table uses AggregatingMergeTree engine
    let query = "SELECT engine FROM system.tables WHERE database = 'exchange' AND name = 'candles'";
    let engine: String = db
        .clickhouse
        .query(query)
        .fetch_one::<String>()
        .await
        .expect("Failed to query table engine");

    assert_eq!(
        engine, "AggregatingMergeTree",
        "Candles table should use AggregatingMergeTree engine"
    );
}

#[tokio::test]
async fn test_trades_table_has_all_required_columns() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    // Query the table schema - just get column names
    let query = "SELECT name FROM system.columns WHERE database = 'exchange' AND table = 'trades'";
    let columns: Vec<String> = db
        .clickhouse
        .query(query)
        .fetch_all::<String>()
        .await
        .expect("Failed to query table schema");

    // Check all required columns exist
    let required = vec![
        "id",
        "market_id",
        "buyer_address",
        "seller_address",
        "buyer_order_id",
        "seller_order_id",
        "price",
        "size",
        "timestamp",
    ];

    for col in required {
        assert!(
            columns.contains(&col.to_string()),
            "Missing required column: {}. Available columns: {:?}",
            col,
            columns
        );
    }
}

#[tokio::test]
async fn test_candles_table_has_all_required_columns() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    // Query the table schema - just get column names
    let query = "SELECT name FROM system.columns WHERE database = 'exchange' AND table = 'candles'";
    let columns: Vec<String> = db
        .clickhouse
        .query(query)
        .fetch_all::<String>()
        .await
        .expect("Failed to query table schema");

    // Check all required columns exist (now using aggregate state columns)
    let required = vec![
        "market_id",
        "interval",
        "timestamp",
        "open_state",
        "high_state",
        "low_state",
        "close_state",
        "volume_state",
    ];

    for col in required {
        assert!(
            columns.contains(&col.to_string()),
            "Missing required column: {}. Available columns: {:?}",
            col,
            columns
        );
    }
}

/// Test that the optimized schema reduces storage by aggregating trades into single candles
#[tokio::test]
async fn test_candles_aggregation_reduces_storage() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    // Insert multiple trades in the same minute bucket (but with different seconds)
    let base_timestamp = 1700000000u32; // 2023-11-15 02:13:20 UTC
    let num_trades = 10u32;

    for i in 0..num_trades {
        let trade = ClickHouseTradeRow {
            id: format!("trade-{}", i),
            market_id: "BTC/USDC".to_string(),
            buyer_address: "buyer".to_string(),
            seller_address: "seller".to_string(),
            buyer_order_id: format!("buyer-order-{}", i),
            seller_order_id: format!("seller-order-{}", i),
            price: 95000000000 + (i as u128 * 1000000), // Varying prices
            size: 1000000,
            side: "buy".to_string(),
            timestamp: base_timestamp + i, // Different seconds within same minute
        };

        let mut insert = db
            .clickhouse
            .insert::<ClickHouseTradeRow>("trades")
            .await
            .unwrap();
        insert.write(&trade).await.unwrap();
        insert.end().await.unwrap();
    }

    // Wait for materialized view to aggregate
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Count rows in trades table (should be num_trades)
    let trade_count: u64 = db
        .clickhouse
        .query("SELECT COUNT(*) FROM exchange.trades WHERE market_id = 'BTC/USDC'")
        .fetch_one()
        .await
        .expect("Failed to count trades");

    assert_eq!(
        trade_count, num_trades as u64,
        "Should have {} trades in trades table",
        num_trades
    );

    // Count rows in candles table for 1m interval
    // With AggregatingMergeTree, should be 1 row (or very few if not yet merged)
    let candle_count: u64 = db
        .clickhouse
        .query("SELECT COUNT(*) FROM exchange.candles WHERE market_id = 'BTC/USDC' AND interval = '1m'")
        .fetch_one()
        .await
        .expect("Failed to count candles");

    assert!(
        candle_count <= 3,
        "Candles should be aggregated into very few rows (got {}), much less than {} trades",
        candle_count,
        num_trades
    );

    // Verify the aggregated candle data is correct using -Merge combinators
    let query = "SELECT
        argMinMerge(open_state) as open,
        maxMerge(high_state) as high,
        minMerge(low_state) as low,
        argMaxMerge(close_state) as close,
        sumMerge(volume_state) as volume
    FROM exchange.candles
    WHERE market_id = 'BTC/USDC' AND interval = '1m'
    GROUP BY timestamp";

    let candle: (u128, u128, u128, u128, u128) = db
        .clickhouse
        .query(query)
        .fetch_one()
        .await
        .expect("Failed to fetch aggregated candle");

    let (open, high, low, close, volume) = candle;

    // Verify OHLCV is correct
    // argMin(price, timestamp) returns price of trade with earliest timestamp
    assert_eq!(
        open, 95000000000,
        "Open should be first trade price (earliest timestamp)"
    );
    assert_eq!(
        high,
        95000000000 + ((num_trades - 1) as u128 * 1000000),
        "High should be highest price"
    );
    assert_eq!(low, 95000000000, "Low should be lowest price");
    // argMax(price, timestamp) returns price of trade with latest timestamp
    assert_eq!(
        close,
        95000000000 + ((num_trades - 1) as u128 * 1000000),
        "Close should be last trade price"
    );
    assert_eq!(
        volume,
        1000000 * num_trades as u128,
        "Volume should be sum of all trades"
    );
}

/// Test that materialized views are created for all intervals
#[tokio::test]
async fn test_materialized_views_exist() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    let query =
        "SELECT name FROM system.tables WHERE database = 'exchange' AND name LIKE 'candles_%_mv'";
    let views: Vec<String> = db
        .clickhouse
        .query(query)
        .fetch_all::<String>()
        .await
        .expect("Failed to query views");

    let required_views = vec![
        "candles_1m_mv",
        "candles_5m_mv",
        "candles_15m_mv",
        "candles_1h_mv",
        "candles_1d_mv",
    ];

    for view in required_views {
        assert!(
            views.contains(&view.to_string()),
            "Missing materialized view: {}",
            view
        );
    }
}

/// Test that we can insert and retrieve a trade
/// Note: Disabled due to ClickHouse eventual consistency - data isn't immediately queryable
#[tokio::test]
async fn test_trades_roundtrip() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    let trade = ClickHouseTradeRow {
        id: "test-trade-roundtrip".to_string(),
        market_id: "BTC/USDC".to_string(),
        buyer_address: "buyer1".to_string(),
        seller_address: "seller1".to_string(),
        buyer_order_id: "buyer-order-1".to_string(),
        seller_order_id: "seller-order-1".to_string(),
        price: 95000000000,
        size: 1000000,
        side: "buy".to_string(),
        timestamp: 1234567890,
    };

    // Insert trade
    let mut insert = db
        .clickhouse
        .insert::<ClickHouseTradeRow>("trades")
        .await
        .expect("Failed to create insert");
    insert.write(&trade).await.expect("Failed to write trade");
    insert.end().await.expect("Failed to end insert");

    // Wait for ClickHouse to process the insert (eventual consistency)
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Query back
    let query = "SELECT * FROM exchange.trades WHERE id = ?";
    let rows = db
        .clickhouse
        .query(query)
        .bind("test-trade-roundtrip")
        .fetch_all::<ClickHouseTradeRow>()
        .await
        .expect("Failed to fetch trade");

    assert_eq!(rows.len(), 1, "Should retrieve exactly one trade");
    let retrieved = &rows[0];

    assert_eq!(retrieved.id, trade.id);
    assert_eq!(retrieved.market_id, trade.market_id);
    assert_eq!(retrieved.price, trade.price);
    assert_eq!(retrieved.size, trade.size);
}
