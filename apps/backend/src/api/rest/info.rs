use axum::{extract::State, response::Json};

use crate::models::api::{InfoErrorResponse, InfoRequest, InfoResponse};

/// Get information about tokens, markets, etc.
#[utoipa::path(
    post,
    path = "/api/info",
    request_body = InfoRequest,
    responses(
        (status = 200, description = "Success", body = InfoResponse),
        (status = 400, description = "Invalid request", body = InfoErrorResponse),
        (status = 404, description = "Resource not found", body = InfoErrorResponse),
        (status = 500, description = "Internal server error", body = InfoErrorResponse)
    ),
    tag = "info"
)]
pub async fn info(
    State(state): State<crate::AppState>,
    Json(request): Json<InfoRequest>,
) -> Result<Json<InfoResponse>, Json<InfoErrorResponse>> {
    match request {
        InfoRequest::TokenDetails { ticker } => {
            // TODO: Implement token details lookup
            // Example: state.db.get_token(&ticker).await
            todo!("Implement token details lookup for ticker: {}", ticker)
        }
        InfoRequest::MarketDetails { market_id } => {
            // TODO: Implement market details lookup
            // Example: state.db.get_market(&market_id).await
            todo!("Implement market details lookup for market_id: {}", market_id)
        }
        InfoRequest::AllMarkets => {
            // TODO: Implement list all markets
            // Example: state.db.list_markets().await
            todo!("Implement list all markets")
        }
        InfoRequest::AllTokens => {
            // TODO: Implement list all tokens
            // Example: state.db.list_tokens().await
            todo!("Implement list all tokens")
        }
    }
}
