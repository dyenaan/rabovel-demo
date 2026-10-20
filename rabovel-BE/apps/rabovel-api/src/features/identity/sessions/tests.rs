use crate::app::router::router_with_state;
use crate::app::state::GatewayStateBuilder;
use crate::app::support::hash_token;
use crate::app::test_support::*;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use identity::security_state::InMemoryEphemeralStore;
use identity::sessions::models::SessionRecord;
use identity::sessions::ports::SessionRepository;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn auth_me_requires_a_live_session_and_logout_revokes_it() {
    let missing = test_app()
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

    let (app, token) = login(test_app()).await;
    let me = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::OK);
    assert_eq!(body_json(me).await["roles"], serde_json::json!(["trader"]));

    let logout = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);
    let rejected = app
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
}
#[tokio::test]
async fn auth_me_rejects_an_expired_session() {
    let ephemeral = Arc::new(InMemoryEphemeralStore::default());
    let token = "expired-token";
    ephemeral
        .create_session(
            &hash_token(token),
            SessionRecord {
                user_id: "usr_expired".to_string(),
                mfa_verified_at: None,
            },
            0,
        )
        .await
        .unwrap();
    let app = router_with_state(GatewayStateBuilder::default().ephemeral(ephemeral).build());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
