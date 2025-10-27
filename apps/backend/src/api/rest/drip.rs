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
            signature: _,
        } => {
            // TODO: Verify signature (skip for dev/test faucet)

            // Parse amount from string to u128
            let amount_value = amount.parse::<u128>().map_err(|_| {
                Json(DripErrorResponse {
                    error: "Invalid amount format".to_string(),
                    code: "INVALID_AMOUNT".to_string(),
                })
            })?;

            // Check token exists
            state.db.get_token(&token_ticker).await.map_err(|e| {
                Json(DripErrorResponse {
                    error: format!("Token not found: {}", e),
                    code: "TOKEN_NOT_FOUND".to_string(),
                })
            })?;

            // Create user if doesn't exist
            let _ = state.db.create_user(user_address.clone()).await;

            // Add balance
            let new_balance = state
                .db
                .add_balance(&user_address, &token_ticker, amount_value)
                .await
                .map_err(|e| {
                    Json(DripErrorResponse {
                        error: format!("Failed to update balance: {}", e),
                        code: "BALANCE_UPDATE_ERROR".to_string(),
                    })
                })?;

            Ok(Json(DripResponse::Faucet {
                user_address,
                token_ticker,
                amount,
                new_balance: new_balance.amount.to_string(),
            }))
        }
    }
}
