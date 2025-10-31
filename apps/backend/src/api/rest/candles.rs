use crate::models::api::{ApiCandle, CandlesRequest, CandlesResponse};
use crate::AppState;
use axum::{extract::State, Json};

/// Get OHLCV candles for a market
///
/// POST /api/candles
#[utoipa::path(
    post,
    path = "/api/candles",
    request_body = CandlesRequest,
    responses(
        (status = 200, description = "Candles retrieved successfully", body = CandlesResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "market-data"
)]
pub async fn get_candles(
    State(state): State<AppState>,
    Json(params): Json<CandlesRequest>,
) -> Result<Json<CandlesResponse>, String> {
    // Validate interval
    if !["1m", "5m", "15m", "1h", "1d"].contains(&params.interval.as_str()) {
        return Err("Invalid interval. Must be one of: 1m, 5m, 15m, 1h, 1d".to_string());
    }

    // Query ClickHouse for candles
    // Materialized views create one row per trade, so aggregate with GROUP BY
    let mut query = format!(
        "SELECT
            toUnixTimestamp(timestamp) as timestamp,
            argMin(open, timestamp) as open,
            max(high) as high,
            min(low) as low,
            argMax(close, timestamp) as close,
            sum(volume) as volume
        FROM exchange.candles
        WHERE market_id = '{}'
          AND interval = '{}'
          AND timestamp >= toDateTime({})
          AND timestamp <= toDateTime({})
        GROUP BY timestamp
        ORDER BY timestamp",
        params.market_id, params.interval, params.from, params.to
    );

    // Handle countBack: limit to N most recent bars
    if let Some(count_back) = params.count_back {
        if count_back > 0 {
            // Get the last N bars by ordering DESC and limiting, then reverse
            query = format!("{} DESC LIMIT {}", query, count_back);
        } else {
            query = format!("{} ASC", query);
        }
    } else {
        query = format!("{} ASC", query);
    }

    let mut candles: Vec<ApiCandle> = state
        .db
        .clickhouse
        .query(&query)
        .fetch_all()
        .await
        .map_err(|e| format!("Failed to query candles: {}", e))?;

    // If we used DESC for countBack, reverse to get ascending order
    if params.count_back.is_some() && params.count_back.unwrap() > 0 {
        candles.reverse();
    }

    Ok(Json(CandlesResponse { candles }))
}
