//! Agent authentication middleware.
//!
//! Client agents authenticate using a Bearer token issued at registration time.
//! The token is stored (as plaintext) in the Client record and verified here.
//!
//! Request format:
//!   Authorization: Bearer {client_id}:{agent_token}
//!
//! The `client_id` prefix allows the server to look up the correct record
//! without scanning all clients.

use crate::repository::client_repository::ClientRepository;
use axum::{
    extract::{Extension, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use std::sync::Arc;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct AuthenticatedAgent {
    pub client_id: String,
}

/// Extract client_id from request URI path for client-specific endpoints.
/// Returns `Some(client_id)` if the path matches `/api/v1/clients/{id}/...`,
/// `None` otherwise.
fn extract_path_client_id(request: &Request) -> Option<&str> {
    let path = request.uri().path();
    // Match pattern: /api/v1/clients/{id}/...
    let prefix = "/api/v1/clients/";
    if let Some(rest) = path.strip_prefix(prefix) {
        rest.split('/').next()
    } else {
        None
    }
}

pub async fn agent_auth_middleware(
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth = match auth_header {
        Some(h) => h,
        None => {
            warn!("Agent request missing Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let raw = auth.token();

    // Expected format: "{client_id}:{token}"
    let (client_id, token) = match raw.split_once(':') {
        Some(parts) => parts,
        None => {
            warn!("Agent token malformed (expected client_id:token)");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // For client-specific endpoints (e.g., hardware push), verify the token's
    // client_id matches the URL path parameter to prevent impersonation.
    if let Some(path_client_id) = extract_path_client_id(&request) {
        if path_client_id != client_id {
            warn!(
                "Agent token client_id '{}' does not match path client_id '{}'",
                client_id, path_client_id
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    match client_repo.verify_agent_token(client_id, token).await {
        Ok(true) => {
            request.extensions_mut().insert(AuthenticatedAgent {
                client_id: client_id.to_string(),
            });
            Ok(next.run(request).await)
        }
        Ok(false) => {
            warn!("Agent token verification failed for client {}", client_id);
            Err(StatusCode::UNAUTHORIZED)
        }
        Err(e) => {
            warn!("DB error during agent auth for {}: {}", client_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use axum::http::Uri;
    use common::models::Client;
    use std::sync::Arc;

    // --- Token verification tests ---

    #[tokio::test]
    async fn test_verify_agent_token_correct() {
        let db = Arc::new(setup_test_db().unwrap());
        let repo = Arc::new(ClientRepository::new(db));

        let mut client = Client::new("test-agent".to_string(), "10.0.0.1".to_string());
        client.id = "agent-001".to_string();
        client.serial_number = Some("SN-AGENT-001".to_string());
        // Store hashed token as real code does
        client.agent_token =
            Some(crate::service::auth_service::hash_token("my-secret-token").unwrap());
        repo.save(&client).await.unwrap();

        let ok = repo
            .verify_agent_token("agent-001", "my-secret-token")
            .await
            .unwrap();
        assert!(ok);
    }

    #[tokio::test]
    async fn test_verify_agent_token_wrong() {
        let db = Arc::new(setup_test_db().unwrap());
        let repo = Arc::new(ClientRepository::new(db));

        let mut client = Client::new("test-agent".to_string(), "10.0.0.1".to_string());
        client.id = "agent-002".to_string();
        client.serial_number = Some("SN-AGENT-002".to_string());
        client.agent_token =
            Some(crate::service::auth_service::hash_token("correct-token").unwrap());
        repo.save(&client).await.unwrap();

        let ok = repo
            .verify_agent_token("agent-002", "wrong-token")
            .await
            .unwrap();
        assert!(!ok);
    }

    #[tokio::test]
    async fn test_verify_agent_token_missing_client() {
        let db = Arc::new(setup_test_db().unwrap());
        let repo = Arc::new(ClientRepository::new(db));

        let ok = repo
            .verify_agent_token("nonexistent", "any-token")
            .await
            .unwrap();
        assert!(!ok);
    }

    #[tokio::test]
    async fn test_verify_agent_token_no_token_stored() {
        let db = Arc::new(setup_test_db().unwrap());
        let repo = Arc::new(ClientRepository::new(db));

        let mut client = Client::new("test-agent".to_string(), "10.0.0.1".to_string());
        client.id = "agent-003".to_string();
        client.serial_number = Some("SN-AGENT-003".to_string());
        repo.save(&client).await.unwrap();

        let ok = repo
            .verify_agent_token("agent-003", "any-token")
            .await
            .unwrap();
        assert!(!ok);
    }

    // --- Path client_id extraction tests ---

    #[test]
    fn test_extract_path_client_id() {
        let req = Request::builder()
            .uri(Uri::from_static("/api/v1/clients/test-123/hardware"))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_path_client_id(&req), Some("test-123"));

        let req = Request::builder()
            .uri(Uri::from_static("/api/v1/clients/test-123/hardware/extra"))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_path_client_id(&req), Some("test-123"));

        let req = Request::builder()
            .uri(Uri::from_static("/api/v1/agent/commands/pending"))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_path_client_id(&req), None);

        let req = Request::builder()
            .uri(Uri::from_static("/api/v1/health"))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_path_client_id(&req), None);

        let req = Request::builder()
            .uri(Uri::from_static("/api/v1/clients//hardware"))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_path_client_id(&req), Some(""));
    }
}
