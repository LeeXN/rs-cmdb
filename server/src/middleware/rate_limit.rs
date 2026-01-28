//! Rate limiting middleware for authentication endpoints
//!
//! Provides rate limiting using the governor crate to prevent brute force attacks.

use axum::{http::StatusCode, response::Response};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Rate limiter for login endpoint (10 requests per minute)
pub type LoginRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Rate limiter for registration endpoint (5 requests per hour)
pub type RegistrationRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Rate limiting configuration
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Login requests allowed per minute
    pub login_requests_per_minute: NonZeroU32,
    /// Registration requests allowed per hour
    pub registration_requests_per_hour: NonZeroU32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            login_requests_per_minute: NonZeroU32::new(10).unwrap(),
            registration_requests_per_hour: NonZeroU32::new(5).unwrap(),
        }
    }
}

/// Create a login rate limiter
pub fn create_login_rate_limiter(config: &RateLimitConfig) -> LoginRateLimiter {
    Arc::new(RateLimiter::direct(Quota::per_minute(
        config.login_requests_per_minute,
    )))
}

/// Create a registration rate limiter
pub fn create_registration_rate_limiter(config: &RateLimitConfig) -> RegistrationRateLimiter {
    Arc::new(RateLimiter::direct(Quota::per_hour(
        config.registration_requests_per_hour,
    )))
}

/// Check if rate limited and return appropriate response
pub fn check_rate_limit(
    limiter: &RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
) -> Result<(), RateLimitResponse> {
    if limiter.check().is_err() {
        Err(RateLimitResponse::new())
    } else {
        Ok(())
    }
}

/// Response when rate limit is exceeded
#[derive(Debug)]
pub struct RateLimitResponse {
    status: StatusCode,
    body: String,
    retry_after: Option<u64>,
}

impl RateLimitResponse {
    pub fn new() -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            body: r#"{"error":"Rate limit exceeded. Please try again later."}"#.to_string(),
            retry_after: Some(60), // Default retry after 60 seconds
        }
    }
}

impl From<RateLimitResponse> for Response {
    fn from(val: RateLimitResponse) -> Self {
        let mut builder = Response::builder().status(val.status);

        if let Some(retry_after) = val.retry_after {
            builder = builder.header("Retry-After", retry_after.to_string());
        }

        builder.body(axum::body::Body::from(val.body)).unwrap()
    }
}
