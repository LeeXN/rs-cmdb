use axum::{response::Json, http::StatusCode};
use serde_json::{json, Value};
use tracing::instrument;

/// Health check endpoint
#[instrument]
pub async fn health_check() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "status": "UP",
        "message": "CMDB server is running"
    })))
}

/// Version information endpoint
#[instrument]
pub async fn version() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME"),
    })))
} 