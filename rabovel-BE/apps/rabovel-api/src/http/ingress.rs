use crate::app::constants::IP_RATE_LIMIT;
use crate::http::ingress;
use crate::ApiError;
use crate::GatewayState;
use axum::extract::ConnectInfo;
use axum::extract::Request;
use axum::extract::State;
use axum::middleware::Next;
use identity::AuthError;
use sha2::Digest;
use sha2::Sha256;
use std::net::IpAddr;
use std::net::SocketAddr;
use subtle::ConstantTimeEq;

pub(crate) fn validate_secret(secret: Option<&str>, host: Option<&str>) -> Result<(), String> {
    match secret {
        Some(value) if value.len() == 64 && value.bytes().all(|c| c.is_ascii_hexdigit()) => Ok(()),
        Some(_) => Err("RABOVEL_GATEWAY_SECRET must be 64 hexadecimal characters".into()),
        None if matches!(host, Some("localhost" | "127.0.0.1" | "[::1]")) => Ok(()),
        None => Err("RABOVEL_GATEWAY_SECRET is required outside localhost".into()),
    }
}

pub(crate) fn client_ip(state: &GatewayState, request: &Request) -> Result<String, ApiError> {
    if let Some(expected) = state.gateway_secret {
        let supplied = request
            .headers()
            .get("x-rabovel-gateway-secret")
            .ok_or_else(|| ApiError::from(AuthError::ForbiddenAction))?;
        let digest: [u8; 32] = Sha256::digest(supplied.as_bytes()).into();
        if !bool::from(expected.ct_eq(&digest)) {
            return Err(ApiError::from(AuthError::ForbiddenAction));
        }
        let ip = request
            .headers()
            .get("x-rabovel-client-ip")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<IpAddr>().ok())
            .ok_or_else(|| {
                ApiError::from(AuthError::PolicyViolation(
                    "invalid gateway client IP".into(),
                ))
            })?;
        return Ok(ip.to_string());
    }
    // Never use caller-controlled forwarded headers in direct development mode.
    Ok(request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(peer)| peer.ip().to_string())
        .unwrap_or_else(|| "unknown".into()))
}

pub(crate) async fn ip_rate_limit_middleware(
    State(state): State<GatewayState>,
    request: Request,
    next: Next,
) -> Result<axum::response::Response, ApiError> {
    // Health is the sole unauthenticated endpoint when gateway mode is enabled.
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }
    let ip = ingress::client_ip(&state, &request)?;
    state
        .rate_limiter
        .check(&format!("ip:{ip}"), IP_RATE_LIMIT.0, IP_RATE_LIMIT.1)
        .await
        .map_err(ApiError::from)?;
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{router_with_state, GatewayStateBuilder};
    use axum::{body::Body, http::StatusCode};
    use tower::ServiceExt;

    const SECRET: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn hosted_origin_requires_a_valid_secret() {
        assert!(validate_secret(None, Some("demo.example.com")).is_err());
        assert!(validate_secret(Some("short"), Some("localhost")).is_err());
        assert!(validate_secret(None, Some("localhost")).is_ok());
        assert!(validate_secret(Some(SECRET), Some("demo.example.com")).is_ok());
    }

    #[tokio::test]
    async fn application_requires_gateway_even_with_a_forged_forwarded_ip() {
        let app = router_with_state(
            GatewayStateBuilder::default()
                .gateway_secret(SECRET)
                .build(),
        );
        for supplied in [None, Some("wrong")] {
            let mut request = Request::builder()
                .uri("/auth/me")
                .header("x-forwarded-for", "203.0.113.1")
                .header("x-rabovel-client-ip", "203.0.113.1");
            if let Some(value) = supplied {
                request = request.header("x-rabovel-gateway-secret", value);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header("x-rabovel-gateway-secret", SECRET)
                    .header("x-rabovel-client-ip", "203.0.113.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Gateway authentication does not replace the user's session.
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn authenticated_ip_is_validated_and_direct_mode_ignores_headers() {
        let state = GatewayStateBuilder::default()
            .gateway_secret(SECRET)
            .build();
        let request = Request::builder()
            .header("x-rabovel-gateway-secret", SECRET)
            .header("x-rabovel-client-ip", "1.2.3.4, 5.6.7.8")
            .body(Body::empty())
            .unwrap();
        assert!(client_ip(&state, &request).is_err());
        let state = GatewayStateBuilder::default().build();
        let mut request = Request::builder()
            .header("x-forwarded-for", "203.0.113.1")
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(ConnectInfo(
            "127.0.0.1:12345".parse::<SocketAddr>().unwrap(),
        ));
        assert_eq!(client_ip(&state, &request).unwrap(), "127.0.0.1");
    }
}
