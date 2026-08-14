use axum::{http::StatusCode, response::Json};
use serde_json::{Value, json};
use tracing::instrument;

use crate::config;

/// Health check endpoint
#[instrument]
pub async fn health_check() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "UP",
            "message": "CMDB server is running"
        })),
    )
}

/// Version information endpoint
#[instrument]
pub async fn version() -> (StatusCode, Json<Value>) {
    let body = if config::get_config().expose_version {
        json!({
            "version": env!("CARGO_PKG_VERSION"),
            "name": env!("CARGO_PKG_NAME"),
        })
    } else {
        json!({
            "version": "unknown",
            "name": "cmdb",
        })
    };
    (StatusCode::OK, Json(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_up() {
        let (status, body) = health_check().await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.0["status"], "UP");
    }

    #[tokio::test]
    async fn test_version_returns_package_info() {
        let (status, body) = version().await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.0["version"].as_str().is_some());
        assert!(body.0["name"].as_str().is_some());
    }
}
