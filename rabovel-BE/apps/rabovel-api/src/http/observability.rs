//! Metadata-only request logs: no bodies, query strings, IPs, headers or errors.

use axum::extract::MatchedPath;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;

pub(crate) async fn observe(request: Request, next: Next) -> Response {
    let started = Instant::now();
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| uuid::Uuid::parse_str(v).ok())
        .unwrap_or_else(uuid::Uuid::new_v4)
        .to_string();
    let method = request.method().as_str().to_string();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str())
        .unwrap_or("<unmatched>")
        .to_string();
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-request-id", request_id.parse().expect("UUID header"));
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert("x-content-type-options", "nosniff".parse().unwrap());
    // Health polling is intentionally omitted to avoid noisy logs.
    if route != "/health" {
        println!(
            "{}",
            serde_json::json!({"event":"http_request","request_id":request_id,"method":method,"route":route,"status":response.status().as_u16(),"duration_ms":started.elapsed().as_millis()})
        );
    }
    response
}

// Provider status objects can be returned with HTTP 200. Redact their optional
// error field as well as non-2xx errors; raw RPC URLs can contain API keys.
pub(crate) fn serialize_provider_error<S: serde::Serializer>(
    error: &Option<String>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match error {
        Some(_) => serializer.serialize_some(
            "The provider operation could not be completed. Check status before retrying.",
        ),
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::StatusCode,
    };
    use tower::ServiceExt;
    #[test]
    fn successful_status_payloads_do_not_expose_provider_errors() {
        #[derive(serde::Serialize)]
        struct Status {
            #[serde(serialize_with = "super::serialize_provider_error")]
            error: Option<String>,
        }
        let json = serde_json::to_string(&Status {
            error: Some("https://rpc.example?api-key=private-key".into()),
        })
        .unwrap();
        assert!(!json.contains("private-key"));
        assert!(json.contains("Check status"));
        assert_eq!(
            serde_json::to_value(Status { error: None }).unwrap()["error"],
            serde_json::Value::Null
        );
    }
    struct DependencyDown;
    #[async_trait::async_trait]
    impl identity::access::rate_limiter::RateLimiter for DependencyDown {
        async fn check(&self, _: &str, _: u32, _: u64) -> Result<(), identity::AuthError> {
            Err(identity::AuthError::ServiceUnavailable)
        }
    }
    #[tokio::test]
    async fn unavailable_security_state_blocks_requests_but_not_liveness() {
        let app = crate::router_with_state(
            crate::GatewayStateBuilder::default()
                .rate_limiter(std::sync::Arc::new(DependencyDown))
                .build(),
        );
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/instruments")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(error["code"], "service_unavailable");
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
    #[tokio::test]
    async fn errors_and_rejected_requests_receive_safe_headers() {
        let app = crate::router_with_state(
            crate::GatewayStateBuilder::default()
                .gateway_secret(&"a".repeat(64))
                .build(),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/auth/me?secret=do-not-log")
                    .header("x-request-id", "injected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(
            uuid::Uuid::parse_str(response.headers()["x-request-id"].to_str().unwrap()).is_ok()
        );
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(response.headers()["x-content-type-options"], "nosniff");
        let response = axum::response::IntoResponse::into_response(crate::ApiError {
            kind: crate::app::errors::FailureKind::Unavailable,
            error: identity::AuthError::PolicyViolation(
                "postgres://user:password@internal-host".into(),
            ),
        });
        let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("password"));
    }
}
