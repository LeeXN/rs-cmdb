///
/// This module contains integration tests for authentication-related API endpoints.
/// Tests use a test server setup that mimics the API router without requiring
/// a full server instance to run.

use std::sync::Arc;
use axum::{
    body::Body,
    extract::{Json, State},
    http::{StatusCode, HeaderValue},
    Router,
};
use serde_json::json;
use tokio::time::{timeout, Duration};
use tower::ServiceBuilder;
use crate::service::auth_service::AuthService;
use common::entity::user::{User, Role};

/// Test application state
#[derive(Clone)]
struct TestState {
    auth_service: Arc<AuthService>,
}

/// Login credentials
#[derive(Clone)]
struct LoginCredentials {
    username: String,
    password: String,
}

/// Login response
#[derive(serde::Serialize, serde::Deserialize)]
struct LoginResponse {
    token: String,
    user: User,
}

/// Creates a test API router with authentication routes
fn create_test_auth_router() -> Router {
    let auth_service = Arc::new(AuthService::new("test_secret_key_for_integration_tests".to_string()));
    let state = TestState { auth_service };

    axum::Router::new()
        .route("/api/auth/login", axum::routing::post(login))
        .route("/api/auth/token", axum::routing::post(generate_test_token))
        .route("/api/auth/logout", axum::routing::post(logout))
        .with_state(state)
}

/// Login endpoint handler
async fn login(
    State(state): State<TestState>,
    Json(creds): Json<LoginCredentials>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let auth_service = &state.auth_service;

    let user = match creds.username.as_str() {
        "admin" => User {
            id: "admin-id".to_string(),
            username: "admin".to_string(),
            password_hash: auth_service.hash_password("admin123").unwrap(),
            role: Role::Admin,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        },
        "testuser" => User {
            id: "test-user-id".to_string(),
            username: "testuser".to_string(),
            password_hash: auth_service.hash_password("user123").unwrap(),
            role: Role::User,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        },
        _ => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let is_valid = match creds.password.as_str() {
        "admin123" if creds.username == "admin" => true,
        "user123" if creds.username == "testuser" => true,
        _ => false,
    };

    if !is_valid {
        return Ok((json!({"error": "Invalid credentials"}), StatusCode::UNAUTHORIZED));
    }

    let token = auth_service.generate_token(&user).unwrap();

    Ok(Json(json!({
        "token": token,
        "user": {
            "id": user.id,
            "username": user.username,
            "role": user.role,
        }
    })))
}

/// Generate token endpoint handler
async fn generate_test_token(
    State(state): State<TestState>,
    Json(creds): Json<LoginCredentials>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    login(State(state), Json(creds)).await
}

/// Logout endpoint handler (no-op for testing)
async fn logout() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({"message": "Logged out"})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::{Path, Request},
        http::{HeaderMap, Method, StatusCode, Uri},
        routing::Router,
    };
    use tower::ServiceExt;
    use futures::future;

    /// Helper to make JSON POST requests
    async fn make_json_request(
        app: &Router,
        path: &str,
        body: serde_json::Value,
    ) -> Result<(StatusCode, serde_json::Value), String> {
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(Uri::from_static(path))
                    .header("content-type", "application/json")
                    .body(Body::from(body)),
            )
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .map_err(|e| format!("Failed to read body: {}", e))?;

        let body: serde_json::Value = serde_json::from_slice(&body_bytes)
            .map_err(|e| format!("Failed to parse body: {}", e))?;

        Ok((response.status(), body))
    }

    /// Helper to extract token from response
    fn extract_token(body: &serde_json::Value) -> Option<String> {
        body.get("token")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
    }

    /// Helper to create login credentials
    fn login_creds(username: &str, password: &str) -> serde_json::Value {
        json!({
            "username": username,
            "password": password,
        })
    }

    #[tokio::test]
    async fn test_login_with_valid_credentials_returns_token() {
        let app = create_test_auth_router();
        let creds = login_creds("admin", "admin123");

        let (status, body) = make_json_request(&app, "/api/auth/login", creds).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_object(), "Response should be object");

        let token = extract_token(&body).expect("Response should contain token");
        assert!(!token.is_empty(), "Token should not be empty");
    }

    #[tokio::test]
    async fn test_login_with_invalid_credentials_returns_401() {
        let app = create_test_auth_router();
        let creds = login_creds("admin", "wrongpassword");

        let (status, body) = make_json_request(&app, "/api/auth/login", creds).await.unwrap();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(body.is_object(), "Response should be object");

        let error = body.get("error").expect("Response should have error");
        assert_eq!(error, "Invalid credentials", "Error message should indicate invalid credentials");
    }

    #[tokio::test]
    async fn test_login_with_nonexistent_user_returns_401() {
        let app = create_test_auth_router();
        let creds = login_creds("nonexistent", "password");

        let (status, _body) = make_json_request(&app, "/api/auth/login", creds).await.unwrap();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_with_missing_password_returns_400() {
        let app = create_test_auth_router();
        let creds = json!({"username": "admin"});

        let (status, _) = make_json_request(&app, "/api/auth/login", creds).await.unwrap();

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_generate_token_with_valid_credentials_returns_token() {
        let app = create_test_auth_router();
        let creds = login_creds("testuser", "user123");

        let (status, body) = make_json_request(&app, "/api/auth/token", creds).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_object(), "Response should be object");

        let token = extract_token(&body).expect("Response should contain token");
        assert!(!token.is_empty(), "Token should not be empty");
    }

    #[tokio::test]
    async fn test_generate_token_with_invalid_credentials_returns_401() {
        let app = create_test_auth_router();
        let creds = login_creds("testuser", "wrongpassword");

        let (status, _body) = make_json_request(&app, "/api/auth/token", creds).await.unwrap();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_logout_returns_success_message() {
        let app = create_test_auth_router();

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(Uri::from_static("/api/auth/logout"))
                    .body(Body::empty()),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_concurrent_login_requests() {
        let app = create_test_auth_router();

        let tasks = (0..5)
            .map(|_i| {
                let app = app.clone();
                let creds = login_creds("admin", "admin123");
                tokio::spawn(async move {
                    make_json_request(&app, "/api/auth/login", creds).await
                })
            })
            .collect::<Vec<_>>();

        let results = futures::future::join_all(tasks).await;

        for result in results {
            let (status, body) = result.unwrap().unwrap();
            assert_eq!(status, StatusCode::OK);
            assert!(extract_token(&body).is_some());
        }
    }

    #[tokio::test]
    async fn test_login_timeout_handling() {
        let app = create_test_auth_router();
        let creds = login_creds("admin", "admin123");

        let result = timeout(
            Duration::from_secs(1),
            make_json_request(&app, "/api/auth/login", creds),
        )
        .await;

        assert!(result.is_ok(), "Login should complete within timeout");
        let (status, body) = result.unwrap().unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(extract_token(&body).is_some());
    }
}
