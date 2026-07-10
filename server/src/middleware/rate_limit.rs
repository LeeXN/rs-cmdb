use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::keyed::DefaultKeyedStateStore,
};
use serde_json::json;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

pub type KeyedRateLimiter = RateLimiter<
    IpAddr,
    DefaultKeyedStateStore<IpAddr>,
    DefaultClock,
    NoOpMiddleware,
>;

pub mod strategies {
    use super::*;

    pub fn login() -> Quota {
        Quota::per_minute(NonZeroU32::new(10).unwrap())
    }

    pub fn register() -> Quota {
        Quota::per_hour(NonZeroU32::new(5).unwrap())
    }

    pub fn change_password() -> Quota {
        Quota::per_hour(NonZeroU32::new(5).unwrap())
    }
}

pub fn make_limiter(quota: Quota) -> Arc<KeyedRateLimiter> {
    Arc::new(RateLimiter::keyed(quota))
}

pub fn extract_client_ip<B>(req: &Request<B>) -> IpAddr {
    if let Some(fwd) = req.headers().get("x-forwarded-for") {
        if let Ok(val) = fwd.to_str() {
            if let Some(ip_str) = val.split(',').next().map(|s| s.trim()) {
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }
    if let Some(ci) = req.extensions().get::<ConnectInfo<IpAddr>>() {
        return ci.0;
    }
    IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))
}

pub async fn rate_limit_middleware(
    State(limiter): State<Arc<KeyedRateLimiter>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_client_ip(&req);
    if limiter.check_key(&ip).is_err() {
        let retry_after = 60u64;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("Retry-After", retry_after.to_string())],
            Json(json!({
                "status": 429,
                "message": "Rate limit exceeded. Please try again later."
            })),
        )
            .into_response();
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_request() -> Request<Body> {
        Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap()
    }

    fn dummy_request_with_ip(ip: &str) -> Request<Body> {
        Request::builder()
            .uri("/test")
            .extension(ConnectInfo(ip.parse::<IpAddr>().unwrap()))
            .body(Body::empty())
            .unwrap()
    }

    fn dummy_request_with_xff(ip: &str) -> Request<Body> {
        Request::builder()
            .uri("/test")
            .header("x-forwarded-for", ip)
            .body(Body::empty())
            .unwrap()
    }

    #[test]
    fn test_extract_client_ip_fallback() {
        let ip = extract_client_ip(&dummy_request());
        assert_eq!(ip, IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
    }

    #[test]
    fn test_extract_client_ip_from_connect_info() {
        let ip = extract_client_ip(&dummy_request_with_ip("10.0.0.1"));
        assert_eq!(ip, "10.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_client_ip_from_xff() {
        let ip = extract_client_ip(&dummy_request_with_xff("192.168.1.1"));
        assert_eq!(ip, "192.168.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_client_ip_xff_preferred() {
        let req = Request::builder()
            .uri("/test")
            .extension(ConnectInfo("10.0.0.1".parse::<IpAddr>().unwrap()))
            .header("x-forwarded-for", "192.168.1.1")
            .body(Body::empty())
            .unwrap();
        let ip = extract_client_ip(&req);
        assert_eq!(ip, "192.168.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_make_limiter_accepts_within_limit() {
        let limiter = make_limiter(strategies::login());
        let ip = "10.0.0.1".parse().unwrap();
        for _ in 0..10 {
            assert!(limiter.check_key(&ip).is_ok());
        }
        assert!(limiter.check_key(&ip).is_err());
    }

    #[test]
    fn test_make_limiter_different_ips_independent() {
        let limiter = make_limiter(strategies::login());
        let ip_a: IpAddr = "10.0.0.1".parse().unwrap();
        let ip_b: IpAddr = "10.0.0.2".parse().unwrap();

        for _ in 0..10 {
            assert!(limiter.check_key(&ip_a).is_ok());
        }
        assert!(limiter.check_key(&ip_a).is_err());

        for _ in 0..10 {
            assert!(limiter.check_key(&ip_b).is_ok());
        }
    }

    #[test]
    fn test_rate_limit_response_429() {
        let resp: Response = (
            StatusCode::TOO_MANY_REQUESTS,
            [("Retry-After", "60")],
            Json(json!({"status": 429, "message": "Rate limit exceeded."})),
        ).into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            resp.headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok()),
            Some("60")
        );
    }
}
