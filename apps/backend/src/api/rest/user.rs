use axum::{extract::State, response::Json};

use crate::models::api::{UserErrorResponse, UserRequest, UserResponse};

/// Get user-specific data (orders, balances, trades)
#[utoipa::path(
    post,
    path = "/api/user",
    request_body = UserRequest,
    responses(
        (status = 200, description = "Success", body = UserResponse),
        (status = 400, description = "Invalid request", body = UserErrorResponse),
        (status = 404, description = "User or resource not found", body = UserErrorResponse),
        (status = 500, description = "Internal server error", body = UserErrorResponse)
    ),
    tag = "user"
)]
pub async fn user(
    State(state): State<crate::AppState>,
    Json(request): Json<UserRequest>,
) -> Result<Json<UserResponse>, Json<UserErrorResponse>> {
    match request {
        UserRequest::Orders {
            user_address,
            market_id,
            status,
            limit,
        } => {
            // TODO: Implement get user orders
            // Example: state.db.get_user_orders(&user_address, market_id.as_deref(), status.as_deref(), limit).await
            todo!(
                "Implement get user orders for address: {}, market: {:?}, status: {:?}, limit: {:?}",
                user_address,
                market_id,
                status,
                limit
            )
        }
        UserRequest::Balances { user_address } => {
            // TODO: Implement get user balances
            // Example: state.db.get_user_balances(&user_address).await
            todo!("Implement get user balances for address: {}", user_address)
        }
        UserRequest::Trades {
            user_address,
            market_id,
            limit,
        } => {
            // TODO: Implement get user trades
            // Example: state.db.get_user_trades(&user_address, market_id.as_deref(), limit).await
            todo!(
                "Implement get user trades for address: {}, market: {:?}, limit: {:?}",
                user_address,
                market_id,
                limit
            )
        }
    }
}
