use axum::{extract::State, response::Json};

use crate::models::api::{DripErrorResponse, DripRequest, DripResponse};

/// Drip tokens to users (testing/development faucet)
#[utoipa::path(
    post,
    path = "/api/drip",
    request_body = DripRequest,
    responses(
        (status = 200, description = "Tokens dripped successfully", body = DripResponse),
        (status = 400, description = "Invalid request parameters", body = DripErrorResponse),
        (status = 401, description = "Invalid signature", body = DripErrorResponse),
        (status = 404, description = "Token not found", body = DripErrorResponse),
        (status = 500, description = "Internal server error", body = DripErrorResponse)
    ),
    tag = "drip"
)]
pub async fn drip(
    State(state): State<crate::AppState>,
    Json(request): Json<DripRequest>,
) -> Result<Json<DripResponse>, Json<DripErrorResponse>> {
    match request {
        DripRequest::Faucet {
            user_address,
            token_ticker,
            amount,
            signature,
        } => {
            // TODO: Implement faucet
            // Steps:
            // 1. Verify signature (if needed for dev/test faucet)
            // 2. Parse amount from string to u128
            // 3. Check token exists
            // 4. Update user balance in database
            // 5. Return new balance
            todo!(
                "Implement faucet: user={}, token={}, amount={}",
                user_address,
                token_ticker,
                amount
            )
        }
    }
}
