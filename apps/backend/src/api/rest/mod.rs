use axum::{routing::get, Router};

pub mod health;

pub fn create_routes() -> Router {
    Router::new().route("/api/health", get(health::health_check))
}
