use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    extract::Extension,
};
use common::entity::user::{User, Role};

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
