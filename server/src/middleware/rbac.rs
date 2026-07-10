use axum::{
    extract::Extension, extract::Request, http::StatusCode, middleware::Next, response::Response,
};
use common::entity::user::{Role, User};

pub async fn require_admin(
    Extension(user): Extension<User>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if user.role != Role::Admin {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(request).await)
}

pub async fn require_user(
    Extension(user): Extension<User>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if user.role == Role::Viewer {
        return Err(StatusCode::FORBIDDEN);
    }
    // Admin and User are allowed
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, routing::get, Router};
    use common::entity::user::Role;
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "ok"
    }

    fn make_request(app: Router, user: User) -> axum::response::Response {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            app.oneshot(
                axum::http::Request::builder()
                    .uri("/test")
                    .extension(user)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
        })
    }

    #[test]
    fn test_admin_passes_require_admin() {
        let user = User {
            id: "admin".into(),
            username: "admin".into(),
            password_hash: "hash".into(),
            role: Role::Admin,
            created_at: String::new(),
            last_login: None,
            is_active: true,
        };
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(require_admin));
        let resp = make_request(app, user);
        assert_eq!(resp.status(), 200);
    }

    #[test]
    fn test_user_blocked_from_require_admin() {
        let user = User {
            id: "user".into(),
            username: "user".into(),
            password_hash: "hash".into(),
            role: Role::User,
            created_at: String::new(),
            last_login: None,
            is_active: true,
        };
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(require_admin));
        let resp = make_request(app, user);
        assert_eq!(resp.status(), 403);
    }

    #[test]
    fn test_viewer_blocked_from_require_admin() {
        let user = User {
            id: "viewer".into(),
            username: "viewer".into(),
            password_hash: "hash".into(),
            role: Role::Viewer,
            created_at: String::new(),
            last_login: None,
            is_active: true,
        };
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(require_admin));
        let resp = make_request(app, user);
        assert_eq!(resp.status(), 403);
    }

    #[test]
    fn test_admin_passes_require_user() {
        let user = User {
            id: "admin".into(),
            username: "admin".into(),
            password_hash: "hash".into(),
            role: Role::Admin,
            created_at: String::new(),
            last_login: None,
            is_active: true,
        };
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(require_user));
        let resp = make_request(app, user);
        assert_eq!(resp.status(), 200);
    }

    #[test]
    fn test_user_passes_require_user() {
        let user = User {
            id: "user".into(),
            username: "user".into(),
            password_hash: "hash".into(),
            role: Role::User,
            created_at: String::new(),
            last_login: None,
            is_active: true,
        };
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(require_user));
        let resp = make_request(app, user);
        assert_eq!(resp.status(), 200);
    }

    #[test]
    fn test_viewer_blocked_from_require_user() {
        let user = User {
            id: "viewer".into(),
            username: "viewer".into(),
            password_hash: "hash".into(),
            role: Role::Viewer,
            created_at: String::new(),
            last_login: None,
            is_active: true,
        };
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(require_user));
        let resp = make_request(app, user);
        assert_eq!(resp.status(), 403);
    }
}
