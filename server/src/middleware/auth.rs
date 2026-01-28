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
    let auth = match auth_header {
        Some(header) => header,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let token = auth.token();

    let claims = match auth_service.verify_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

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
