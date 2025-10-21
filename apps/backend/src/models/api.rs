use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse {
    pub message: String,
    pub timestamp: u64,
}
