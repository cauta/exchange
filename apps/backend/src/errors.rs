use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExchangeError {
    // Business logic errors (4xx)
    #[error("Token '{ticker}' does not exist")]
    TokenNotFound { ticker: String },

    #[error("Market '{market_id}' does not exist")]
    MarketNotFound { market_id: String },

    #[error("Market '{market_id}' already exists")]
    MarketAlreadyExists { market_id: String },

    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Order not found")]
    OrderNotFound,

    #[error("User '{address}' not found")]
    UserNotFound { address: String },

    // Infrastructure errors (5xx) - auto-converted
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("ClickHouse error: {0}")]
    ClickHouse(#[from] clickhouse::error::Error),
}

pub type Result<T> = std::result::Result<T, ExchangeError>;

impl IntoResponse for ExchangeError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            // Client errors - expose detailed message
            ExchangeError::TokenNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            ExchangeError::MarketNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            ExchangeError::MarketAlreadyExists { .. } => (StatusCode::CONFLICT, self.to_string()),
            ExchangeError::InvalidParameter { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            ExchangeError::InsufficientBalance => (StatusCode::BAD_REQUEST, self.to_string()),
            ExchangeError::OrderNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ExchangeError::UserNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),

            // Server errors - don't leak internal details
            ExchangeError::Database(ref e) => {
                log::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            ExchangeError::ClickHouse(ref e) => {
                log::error!("ClickHouse error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
