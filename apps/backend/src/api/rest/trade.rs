use axum::{extract::State, response::Json};

use crate::models::api::{TradeErrorResponse, TradeRequest, TradeResponse};

/// Execute trades (place/cancel orders)
#[utoipa::path(
    post,
    path = "/api/trade",
    request_body = TradeRequest,
    responses(
        (status = 200, description = "Success", body = TradeResponse),
        (status = 400, description = "Invalid request parameters", body = TradeErrorResponse),
        (status = 401, description = "Invalid signature", body = TradeErrorResponse),
        (status = 404, description = "Order not found", body = TradeErrorResponse),
        (status = 500, description = "Internal server error", body = TradeErrorResponse)
    ),
    tag = "trade"
)]
pub async fn trade(
    State(state): State<crate::AppState>,
    Json(request): Json<TradeRequest>,
) -> Result<Json<TradeResponse>, Json<TradeErrorResponse>> {
    match request {
        TradeRequest::PlaceOrder {
            user_address,
            market_id,
            side,
            order_type,
            price,
            size,
            signature,
        } => {
            // TODO: Implement place order
            // Steps:
            // 1. Verify signature
            // 2. Parse price and size from strings to u128
            // 3. Create Order struct
            // 4. Send to matching engine via state.engine_tx
            // 5. Return order and any immediate trades
            todo!(
                "Implement place order: user={}, market={}, side={:?}, type={:?}, price={}, size={}",
                user_address,
                market_id,
                side,
                order_type,
                price,
                size
            )
        }
        TradeRequest::CancelOrder {
            user_address,
            order_id,
            signature,
        } => {
            // TODO: Implement cancel order
            // Steps:
            // 1. Verify signature
            // 2. Parse order_id from string to Uuid
            // 3. Send cancel request to matching engine
            // 4. Return cancelled order_id
            todo!(
                "Implement cancel order: user={}, order_id={}",
                user_address,
                order_id
            )
        }
    }
}
