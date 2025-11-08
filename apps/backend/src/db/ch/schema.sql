-- Create database if it doesn't exist
CREATE DATABASE IF NOT EXISTS exchange;

-- Trades table for tick data (raw trades from the matching engine)
-- This is the source of truth - all trades are inserted here
CREATE TABLE IF NOT EXISTS exchange.trades (
    id String,
    market_id String,
    buyer_address String,
    seller_address String,
    buyer_order_id String,
    seller_order_id String,
    price UInt128,
    size UInt128,
    side String,
    timestamp DateTime
) ENGINE = MergeTree()
ORDER BY (market_id, timestamp)
PRIMARY KEY (market_id, timestamp);

-- Candles table for pre-aggregated OHLCV data
-- Uses AggregatingMergeTree to store aggregate states and automatically merge them
-- This table stores ONE row per (market_id, interval, timestamp) bucket
-- Storage reduction: 1000 trades/min = 1 candle row instead of 1000 rows
CREATE TABLE IF NOT EXISTS exchange.candles (
    market_id String,
    interval String,
    timestamp DateTime,
    open_state AggregateFunction(argMin, UInt128, DateTime),
    high_state AggregateFunction(max, UInt128),
    low_state AggregateFunction(min, UInt128),
    close_state AggregateFunction(argMax, UInt128, DateTime),
    volume_state AggregateFunction(sum, UInt128)
) ENGINE = AggregatingMergeTree()
ORDER BY (market_id, interval, timestamp)
PRIMARY KEY (market_id, interval, timestamp);

-- Materialized views that aggregate trades into candles on INSERT
-- Each view handles a different time interval
-- The GROUP BY ensures proper aggregation at insert time

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_1m_mv
TO exchange.candles
AS SELECT
    t.market_id,
    '1m' as interval,
    toStartOfMinute(t.timestamp) as timestamp,
    argMinState(t.price, t.timestamp) as open_state,
    maxState(t.price) as high_state,
    minState(t.price) as low_state,
    argMaxState(t.price, t.timestamp) as close_state,
    sumState(t.size) as volume_state
FROM exchange.trades AS t
GROUP BY t.market_id, interval, timestamp;

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_5m_mv
TO exchange.candles
AS SELECT
    t.market_id,
    '5m' as interval,
    toStartOfInterval(t.timestamp, INTERVAL 5 MINUTE) as timestamp,
    argMinState(t.price, t.timestamp) as open_state,
    maxState(t.price) as high_state,
    minState(t.price) as low_state,
    argMaxState(t.price, t.timestamp) as close_state,
    sumState(t.size) as volume_state
FROM exchange.trades AS t
GROUP BY t.market_id, interval, timestamp;

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_15m_mv
TO exchange.candles
AS SELECT
    t.market_id,
    '15m' as interval,
    toStartOfInterval(t.timestamp, INTERVAL 15 MINUTE) as timestamp,
    argMinState(t.price, t.timestamp) as open_state,
    maxState(t.price) as high_state,
    minState(t.price) as low_state,
    argMaxState(t.price, t.timestamp) as close_state,
    sumState(t.size) as volume_state
FROM exchange.trades AS t
GROUP BY t.market_id, interval, timestamp;

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_1h_mv
TO exchange.candles
AS SELECT
    t.market_id,
    '1h' as interval,
    toStartOfHour(t.timestamp) as timestamp,
    argMinState(t.price, t.timestamp) as open_state,
    maxState(t.price) as high_state,
    minState(t.price) as low_state,
    argMaxState(t.price, t.timestamp) as close_state,
    sumState(t.size) as volume_state
FROM exchange.trades AS t
GROUP BY t.market_id, interval, timestamp;

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_1d_mv
TO exchange.candles
AS SELECT
    t.market_id,
    '1d' as interval,
    toStartOfDay(t.timestamp) as timestamp,
    argMinState(t.price, t.timestamp) as open_state,
    maxState(t.price) as high_state,
    minState(t.price) as low_state,
    argMaxState(t.price, t.timestamp) as close_state,
    sumState(t.size) as volume_state
FROM exchange.trades AS t
GROUP BY t.market_id, interval, timestamp;
