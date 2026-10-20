use crate::app::constants::IP_RATE_LIMIT;
use crate::app::router::fail_closed_router;
use crate::app::router::router;
use crate::app::test_support::*;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let response = router()
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
async fn fail_closed_router_preserves_configured_cors_origin() {
    let response = fail_closed_router("http://localhost:3001")
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/auth/me")
                .header("origin", "http://localhost:3001")
                .header("access-control-request-method", "DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "http://localhost:3001"
    );
    assert!(response
        .headers()
        .get("access-control-allow-methods")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("DELETE"));
}
#[tokio::test]
async fn ip_rate_limit_blocks_after_the_configured_window() {
    let app = test_app();
    for _ in 0..IP_RATE_LIMIT.0 {
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
        assert_eq!(response.status(), StatusCode::OK);
    }
    let throttled = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/instruments")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(throttled.status(), StatusCode::TOO_MANY_REQUESTS);
    let health = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
}
