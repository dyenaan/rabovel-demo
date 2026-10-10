//! Public signup and authorization regressions. No external services required.
use super::*;
use axum::body::{to_bytes, Body};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn call(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> axum::response::Response {
    let mut request = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    app.clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}
async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap()
}
fn signup(email: &str, kind: &str) -> Value {
    json!({"display_name":"Demo User","email":email,"password":"correct-horse-battery","account_type":kind})
}
async fn register(app: &Router, email: &str, kind: &str) -> String {
    let response = call(app, "POST", "/auth/register", None, signup(email, kind)).await;
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await["access_token"]
        .as_str()
        .unwrap()
        .to_string()
}
fn organization() -> Value {
    json!({"legal_name":"Demo Issuer Ltd","organization_type":"company","registration_number":"RC-123","jurisdiction":"Nigeria","registered_address":"1 Demo Street","representative_name":"Ada Demo","representative_title":"Director","document_reference":"demo.pdf","beneficial_owners_confirmed":true,"information_certified":true})
}
fn draft() -> Value {
    json!({"instrument_code":"RABO-NG-TELCO","name":"Demo Equity","ticker":"RABO-NG-TELCO","market":"NGX","share_class":"Ordinary","asset_type":"equity","decimals":0,"authorized_units":"100","settlement_currency":"CNGN","representation":"1 token = 1 simulated share","rights_description":"Simulated ordinary-share entitlement.","disclosure":domain::assets::DEMO_DISCLOSURE,"metadata":{"name":"Demo Equity","symbol":"RABO-NG-TELCO","description":"Demo equity","image_uri":null,"external_url":null,"metadata_uri":null,"additional_metadata":[]}})
}

#[tokio::test]
async fn public_signup_roles_survive_login_and_do_not_grant_other_roles() {
    let app = router_with_state(GatewayStateBuilder::default().build());
    for kind in ["investor", "issuer"] {
        let email = format!("{kind}@example.com");
        let token = register(&app, &email, kind).await;
        let expected = if kind == "issuer" { "issuer" } else { "trader" };
        let me = json_body(call(&app, "GET", "/auth/me", Some(&token), Value::Null).await).await;
        assert_eq!(me["roles"], json!([expected]));
        let overview = call(&app, "GET", "/issuer/overview", Some(&token), Value::Null).await;
        assert_eq!(
            overview.status(),
            if kind == "issuer" {
                StatusCode::OK
            } else {
                StatusCode::FORBIDDEN
            }
        );
        if kind == "issuer" {
            assert_eq!(
                json_body(overview).await["account_status"],
                "organization_onboarding_pending"
            );
            assert_eq!(
                call(&app, "POST", "/issuer/assets", Some(&token), draft())
                    .await
                    .status(),
                StatusCode::FORBIDDEN
            );
            assert_eq!(
                call(
                    &app,
                    "GET",
                    "/wallet/payment-assets/cngn",
                    Some(&token),
                    Value::Null
                )
                .await
                .status(),
                StatusCode::FORBIDDEN
            );
        }
        if kind == "investor" {
            for (path, body) in [
                ("/issuer/onboarding", organization()),
                ("/issuer/assets", draft()),
                ("/payment-assets/cngn", Value::Null),
            ] {
                assert_eq!(
                    call(&app, "POST", path, Some(&token), body).await.status(),
                    StatusCode::FORBIDDEN,
                    "{path}"
                );
            }
        }

        let login = call(
            &app,
            "POST",
            "/auth/login",
            None,
            json!({"email":email,"password":"correct-horse-battery"}),
        )
        .await;
        assert_eq!(login.status(), StatusCode::OK);
        let token = json_body(login).await["access_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let me = json_body(call(&app, "GET", "/auth/me", Some(&token), Value::Null).await).await;
        assert_eq!(me["roles"], json!([expected]));
    }
}

#[tokio::test]
async fn public_signup_rejects_privileged_roles_multiple_roles_and_role_injection() {
    let app = router_with_state(GatewayStateBuilder::default().build());
    let mut requests = Vec::new();
    for kind in [
        "admin",
        "compliance_officer",
        "risk_ops",
        "trader",
        "ISSUER",
        "",
    ] {
        requests.push(signup("attacker@example.com", kind));
    }
    for (key, value) in [
        ("roles", json!(["admin"])),
        ("role", json!("admin")),
        ("permissions", json!(["sign:withdrawal"])),
        ("user_id", json!("victim")),
    ] {
        let mut body = signup("attacker@example.com", "issuer");
        body[key] = value;
        requests.push(body);
    }
    let mut body = signup("attacker@example.com", "issuer");
    body["account_type"] = json!(["investor", "issuer"]);
    requests.push(body);
    let mut body = signup("attacker@example.com", "issuer");
    body.as_object_mut().unwrap().remove("account_type");
    requests.push(body);
    for body in requests {
        let response = call(&app, "POST", "/auth/register", None, body).await;
        // Axum reports malformed enum representations as 400, and invalid
        // values/unknown fields as 422. Both must fail before creating a user.
        assert!(matches!(
            response.status(),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
        ));
    }
    assert_eq!(
        call(
            &app,
            "POST",
            "/auth/login",
            None,
            json!({"email":"attacker@example.com","password":"correct-horse-battery"})
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn issuers_cannot_access_or_mutate_another_issuers_asset() {
    let app = router_with_state(GatewayStateBuilder::default().build());
    let owner = register(&app, "owner@example.com", "issuer").await;
    let other = register(&app, "other@example.com", "issuer").await;
    for token in [&owner, &other] {
        assert_eq!(
            call(
                &app,
                "POST",
                "/issuer/onboarding",
                Some(token),
                organization()
            )
            .await
            .status(),
            StatusCode::OK
        );
    }
    let created = call(&app, "POST", "/issuer/assets", Some(&owner), draft()).await;
    assert_eq!(created.status(), StatusCode::OK);
    let asset = json_body(created).await;
    let id = asset["asset_id"].as_str().unwrap();
    assert_eq!(
        json_body(call(&app, "GET", "/issuer/assets", Some(&other), Value::Null).await).await,
        json!([])
    );
    for (method, suffix, body) in [
        ("GET", "", Value::Null),
        ("PATCH", "", draft()),
        ("DELETE", "", Value::Null),
        ("POST", "/submit", Value::Null),
        ("GET", "/setup", Value::Null),
        ("POST", "/setup", Value::Null),
        ("GET", "/inventory", Value::Null),
        ("POST", "/inventory", Value::Null),
        (
            "POST",
            "/backing",
            json!({"summary":"Demo backing evidence","document_name":"backing.pdf","content_type":"application/pdf","size_bytes":100}),
        ),
        ("POST", "/listing", Value::Null),
        (
            "POST",
            "/image-upload",
            json!({"content_type":"image/png","size":100}),
        ),
        (
            "POST",
            "/image-upload/confirm",
            json!({"object_path":format!("assets/{id}/images/image.png"),"content_type":"image/png"}),
        ),
    ] {
        let response = call(
            &app,
            method,
            &format!("/issuer/assets/{id}{suffix}"),
            Some(&other),
            body,
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "{method} {suffix}"
        );
    }
    let unchanged = call(
        &app,
        "GET",
        &format!("/issuer/assets/{id}"),
        Some(&owner),
        Value::Null,
    )
    .await;
    assert_eq!(unchanged.status(), StatusCode::OK);
    assert_eq!(json_body(unchanged).await, asset);
}
