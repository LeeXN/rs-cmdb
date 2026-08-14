use crate::repository::user_repository::UserRepository;
use crate::service::auth_service::AuthService;
use axum::{
    extract::Extension, extract::Request, http::StatusCode, middleware::Next, response::Response,
};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use std::sync::Arc;
use tracing::error;

pub async fn auth_middleware(
    Extension(auth_service): Extension<Arc<AuthService>>,
    Extension(user_repo): Extension<Arc<UserRepository>>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = if let Some(header) = auth_header {
        header.token().to_string()
    } else {
        let query = request.uri().query().unwrap_or_default();
        let Some(raw_token) = query.split('&').find_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some("access_token"), Some(value)) => Some(value),
                _ => None,
            }
        }) else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        urlencoding::decode(raw_token)
            .map(|value| value.into_owned())
            .map_err(|_| StatusCode::UNAUTHORIZED)?
    };

    let claims = match auth_service.verify_token(&token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Check blacklist
    if auth_service.check_blacklist(&claims.jti).await.is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user = match user_repo.get(&claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(e) => {
            error!("Database error in auth middleware: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if !user.is_active {
        return Err(StatusCode::FORBIDDEN);
    }

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::redb_store::RedbStore;
    use axum::{Router, body::Body, routing::get};
    use common::entity::user::{Role, User};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn setup_test_env() -> (Arc<RedbStore>, Arc<AuthService>, Arc<UserRepository>) {
        let path = std::env::temp_dir().join(format!("auth_test_{}.db", rand::random::<u64>()));
        let _ = std::fs::remove_file(&path);
        let db = Arc::new(RedbStore::new(&path).unwrap());
        let auth_service = Arc::new(AuthService::new("test_jwt_secret".to_string()));
        let user_repo = Arc::new(UserRepository::new(db.clone()));
        (db, auth_service, user_repo)
    }

    async fn auth_handler() -> &'static str {
        "authenticated"
    }

    #[tokio::test]
    async fn test_valid_token_passes() {
        let (_, auth_service, user_repo) = setup_test_env();
        let user = User {
            id: "test-id".into(),
            username: "testuser".into(),
            password_hash: "hash".into(),
            role: Role::Admin,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        };
        user_repo.save(&user).await.unwrap();
        let token = auth_service.generate_token(&user).unwrap();

        let app = Router::new()
            .route("/test", get(auth_handler))
            .layer(axum::middleware::from_fn(auth_middleware))
            .layer(Extension(auth_service))
            .layer(Extension(user_repo));

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/test")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_missing_header_returns_unauthorized() {
        let (_, auth_service, user_repo) = setup_test_env();

        let app = Router::new()
            .route("/test", get(auth_handler))
            .layer(axum::middleware::from_fn(auth_middleware))
            .layer(Extension(auth_service))
            .layer(Extension(user_repo));

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
    }

    #[tokio::test]
    async fn test_invalid_token_returns_unauthorized() {
        let (_, auth_service, user_repo) = setup_test_env();

        let app = Router::new()
            .route("/test", get(auth_handler))
            .layer(axum::middleware::from_fn(auth_middleware))
            .layer(Extension(auth_service))
            .layer(Extension(user_repo));

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/test")
                    .header("authorization", "Bearer invalid_token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
    }

    #[tokio::test]
    async fn test_inactive_user_returns_forbidden() {
        let (_, auth_service, user_repo) = setup_test_env();
        let user = User {
            id: "inactive-id".into(),
            username: "inactive".into(),
            password_hash: "hash".into(),
            role: Role::Admin,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_login: None,
            is_active: false,
        };
        user_repo.save(&user).await.unwrap();
        let token = auth_service.generate_token(&user).unwrap();

        let app = Router::new()
            .route("/test", get(auth_handler))
            .layer(axum::middleware::from_fn(auth_middleware))
            .layer(Extension(auth_service))
            .layer(Extension(user_repo));

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/test")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 403);
    }
}
