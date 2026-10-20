use crate::app::test_support::*;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn merged_market_and_portfolio_routes_return_frontend_envelopes() {
    let (app, token) = login(test_app()).await;
    for path in [
        "/assets",
        "/markets",
        "/portfolio/summary",
        "/portfolio/holdings",
        "/portfolio/performance?range=1M",
        "/portfolio/activity",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let body = body_json(response).await;
        assert!(body.get("data").is_some(), "{path}: {body}");
    }
    for path in [
        "/portfolio/summary",
        "/portfolio/holdings",
        "/portfolio/performance",
        "/portfolio/activity",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{path}");
    }
}
