use crate::app::test_support::*;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn wallet_authorization_requires_a_server_session() {
    let request = serde_json::json!({
        "wallet_id": "wal_attacker",
        "action": "sign:withdrawal"
    });
    let response = test_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/authorize")
                .header("content-type", "application/json")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
#[tokio::test]
async fn old_forged_principal_and_verified_wallet_are_rejected() {
    let (app, token) = login(test_app()).await;
    let forged = serde_json::json!({
        "principal": {
            "user_id": "attacker",
            "roles": ["admin"],
            "permissions": ["sign:withdrawal"]
        },
        "wallet": {
            "chain": "ethereum",
            "address": "0x1111111111111111111111111111111111111111",
            "provider": "meta_mask",
            "verified": true
        },
        "action": "sign:withdrawal"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/authorize")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(forged.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
