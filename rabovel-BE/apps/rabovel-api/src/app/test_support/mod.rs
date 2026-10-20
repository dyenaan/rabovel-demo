use crate::app::persistence::Persistence;
use crate::app::router::router_with_state;
use crate::app::state::GatewayStateBuilder;
use axum::body::to_bytes;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use axum::Router;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use identity::BrokerageAuthService;
use identity::BrokerageSecurityPolicy;
use identity::UserRole;
use issuer_assets::assets::metadata::providers::storage::InMemoryMetadataStorage;
use std::sync::Arc;
use tower::ServiceExt;

pub(crate) mod chain;
pub(crate) mod wallets;
pub(crate) use chain::{FakeMintSetup, FakePaymentAssets};
pub(crate) use wallets::FakeWalletProof;
pub(crate) fn test_app() -> Router {
    router_with_state(
        GatewayStateBuilder::default()
            .persistence(fixture_repository(UserRole::Trader))
            .wallet_proof(Arc::new(FakeWalletProof))
            .build(),
    )
}

pub(crate) fn test_app_with_role(role: UserRole) -> Router {
    router_with_state(
        GatewayStateBuilder::default()
            .persistence(fixture_repository(role))
            .wallet_proof(Arc::new(FakeWalletProof))
            .metadata_storage(Arc::new(InMemoryMetadataStorage))
            .mint_setup(Arc::new(FakeMintSetup::default()))
            .payment_assets(Arc::new(FakePaymentAssets))
            .security_policy(test_security_policy())
            .build(),
    )
}

pub(crate) fn test_security_policy() -> BrokerageSecurityPolicy {
    BrokerageSecurityPolicy::for_testing()
}
pub(crate) async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
pub(crate) async fn login(app: Router) -> (Router, String) {
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "email": "fixture@example.com",
                "password": "correct-horse-battery"
            })
            .to_string(),
        ))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    (app, body["access_token"].as_str().unwrap().to_owned())
}
pub(crate) async fn register_password_user(app: Router, email: &str) -> (Router, String) {
    let payload = serde_json::json!({
        "display_name": "Demo Investor",
        "account_type": "investor",
        "email": email,
        "password": "correct-horse-battery"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    (app, body["access_token"].as_str().unwrap().to_string())
}
pub(crate) async fn link_phantom_wallet(app: &Router, token: &str) {
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[9_u8; 32]);
    let address = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    let challenge = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/solana/challenges")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "address": address }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let challenge = body_json(challenge).await;
    let signature = ed25519_dalek::Signer::sign(
        &signing_key,
        challenge["message"].as_str().unwrap().as_bytes(),
    );
    let linked = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/link")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "challenge_id": challenge["challenge_id"],
                        "signature": BASE64.encode(signature.to_bytes())
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(linked.status(), StatusCode::OK);
}
pub(crate) fn fixture_repository(role: UserRole) -> Arc<Persistence> {
    static PASSWORD: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let password_hash = PASSWORD
        .get_or_init(|| {
            use argon2::{
                password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
                Argon2,
            };
            Argon2::default()
                .hash_password(b"correct-horse-battery", &SaltString::generate(&mut OsRng))
                .unwrap()
                .to_string()
        })
        .clone();
    let profile = BrokerageAuthService::new(test_security_policy())
        .profile_from_email_password("usr_fixture", "fixture@example.com")
        .unwrap();
    let mut permissions = std::collections::HashSet::from([identity::WalletAction::ViewPortfolio]);
    if role == UserRole::Trader {
        permissions.insert(identity::WalletAction::TradeSpot);
    }
    let principal = identity::AuthPrincipal::new(
        "usr_fixture",
        std::collections::HashSet::from([role]),
        permissions,
    );
    let mut persistence = Persistence::memory();
    persistence.accounts = Arc::new(
        identity::accounts::repositories::memory::InMemoryAccounts::with_account(
            identity::PasswordCredential {
                user_id: "usr_fixture".into(),
                email: "fixture@example.com".into(),
                display_name: "Fixture User".into(),
                password_hash,
            },
            profile,
            principal,
        ),
    );
    Arc::new(persistence)
}
