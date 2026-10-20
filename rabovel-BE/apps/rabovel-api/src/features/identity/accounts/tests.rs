use crate::app::test_support::*;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use identity::UserRole;
use tower::ServiceExt;

#[tokio::test]
async fn old_self_asserted_google_payload_is_rejected() {
    let payload = serde_json::json!({
        "user_id": "attacker",
        "email_verified": true,
        "google_subject": "forged-subject"
    });
    let response = test_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/google/callback")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
#[tokio::test]
async fn callback_rejects_client_supplied_roles() {
    let payload = serde_json::json!({
        "code": "valid-code",
        "state": "test-state",
        "role": "admin"
    });
    let response = test_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/google/callback")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
#[tokio::test]
async fn password_registration_and_login_do_not_elevate_admin_email() {
    let app = test_app();
    let (app, investor_token) = register_password_user(app, "Investor@Example.com").await;
    let investor_me = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("authorization", format!("Bearer {investor_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        body_json(investor_me).await["roles"],
        serde_json::json!(["trader"])
    );

    let (app, admin_token) = register_password_user(app, "admin@example.com").await;
    let admin_me = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        body_json(admin_me).await["roles"],
        serde_json::json!(["trader"])
    );

    let login =
        serde_json::json!({"email": "investor@example.com", "password": "correct-horse-battery"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
#[tokio::test]
async fn password_auth_rejects_wrong_password_and_role_fields() {
    let (app, _) = register_password_user(test_app(), "user@example.com").await;
    let wrong = serde_json::json!({"email": "user@example.com", "password": "wrong-password"});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(wrong.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let forged = serde_json::json!({"display_name": "Attacker", "email": "attacker@example.com", "password": "password123", "role": "admin"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(forged.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
#[tokio::test]
async fn stored_roles_are_authoritative_on_issuer_endpoints() {
    let (issuer_app, issuer_token) = login(test_app_with_role(UserRole::Issuer)).await;
    let overview = issuer_app
        .oneshot(
            Request::builder()
                .uri("/issuer/overview")
                .header("authorization", format!("Bearer {issuer_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(overview.status(), StatusCode::OK);

    let (investor_app, investor_token) = login(test_app()).await;
    let forbidden = investor_app
        .oneshot(
            Request::builder()
                .uri("/issuer/overview")
                .header("authorization", format!("Bearer {investor_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let (admin_app, admin_token) = login(test_app_with_role(UserRole::Admin)).await;
    let me = admin_app
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(body_json(me).await["roles"], serde_json::json!(["admin"]));
}
#[tokio::test]
async fn retired_routes_have_no_registered_handlers() {
    let app = test_app();
    for (method, path) in [
        ("GET", "/auth/google/start"),
        ("POST", "/auth/google/callback"),
        ("POST", "/tokenization/requests"),
        ("GET", "/tokenization/requests/tok_old"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{method} {path}");
    }
}
