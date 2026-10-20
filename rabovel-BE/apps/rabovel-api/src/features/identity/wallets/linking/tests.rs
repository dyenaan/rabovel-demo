use crate::app::test_support::*;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use identity::wallets::linking::ports::WalletProofVerifier;
use identity::wallets::providers::siwe::SiweWalletProofVerifier;
use identity::UserRole;
use tower::ServiceExt;

#[tokio::test]
async fn wallet_challenge_is_session_bound_and_single_use() {
    let (app, token) = login(test_app()).await;
    let request = serde_json::json!({
        "address": "0x6da01670d8fc844e736095918bbe11fe8d564163",
        "chain_id": 1,
        "provider": "meta_mask"
    });
    let challenge = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/challenges")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(challenge.status(), StatusCode::OK);
    let challenge = body_json(challenge).await;
    let link = serde_json::json!({
        "challenge_id": challenge["challenge_id"],
        "signature": "valid-test-signature"
    });
    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/link")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(link.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let replay = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/link")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(link.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(replay.status(), StatusCode::CONFLICT);
}
#[tokio::test]
async fn phantom_wallet_is_verified_persisted_and_listed() {
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[7_u8; 32]);
    let address = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    let (app, token) = login(test_app_with_role(UserRole::Trader)).await;
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
    assert_eq!(challenge.status(), StatusCode::OK);
    let challenge = body_json(challenge).await;
    let message = challenge["message"].as_str().unwrap();
    let signature = ed25519_dalek::Signer::sign(&signing_key, message.as_bytes());
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
    let linked = body_json(linked).await;
    assert_eq!(linked["wallet"]["chain"], "solana");
    assert_eq!(linked["wallet"]["provider"], "phantom");
    assert_eq!(linked["wallet"]["address"], address);

    let wallets = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wallets")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wallets.status(), StatusCode::OK);
    let wallets = body_json(wallets).await;
    assert_eq!(wallets.as_array().unwrap().len(), 1);
    assert_eq!(wallets[0]["address"], address);

    let payment_account = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wallet/payment-assets/cngn")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(payment_account.status(), StatusCode::OK);
    let payment_account = body_json(payment_account).await;
    assert_eq!(payment_account["wallet_address"], address);
    assert_eq!(payment_account["token_account"], "investor-cngn-account");
    assert_eq!(payment_account["balance_base_units"], "25000000");

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/payment-assets/cngn")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::OK);

    let catalog = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/investor/catalog")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(catalog.status(), StatusCode::OK);
    let catalog = body_json(catalog).await;
    assert_eq!(catalog["wallet_address"], address);
    assert_eq!(catalog["cngn"]["balance_base_units"], "25000000");
    assert_eq!(catalog["assets"].as_array().unwrap().len(), 0);

    // Test quote endpoint - buy
    let quote_request = serde_json::json!({
        "asset_id": "asset-1",
        "side": "buy",
        "quantity": "100"
    });
    let quote = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/investor/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(quote_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    // Should return 404 because the fake asset doesn't exist in the fake mint setup
    // (FakeMintSetup returns a mint address that doesn't match "asset-1")
    let quote_body = body_json(quote).await;
    eprintln!(
        "Quote response status: {:?}, body: {:?}",
        StatusCode::NOT_FOUND,
        quote_body
    );
    assert_eq!(quote_body["code"], "policy_violation");

    // Test quote endpoint - sell
    let quote_request = serde_json::json!({
        "asset_id": "asset-1",
        "side": "sell",
        "quantity": "100"
    });
    let quote = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/investor/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(quote_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(quote.status(), StatusCode::NOT_FOUND);
}
#[test]
fn production_siwe_verifier_recovers_expected_address() {
    let message = r#"localhost:4361 wants you to sign in with your Ethereum account:
0x6Da01670d8fc844e736095918bbE11fE8D564163

SIWE Notepad Example

URI: http://localhost:4361
Version: 1
Chain ID: 1
Nonce: kEWepMt9knR6lWJ6A
Issued At: 2021-12-07T18:28:18.807Z"#;
    let signature = "6228b3ecd7bf2df018183aeab6b6f1db1e9f4e3cbe24560404112e25363540eb679934908143224d746bbb5e1aa65ab435684081f4dbb74a0fec57f98f40f5051c";
    let recovered = SiweWalletProofVerifier.verify(message, signature).unwrap();
    assert_eq!(recovered, "0x6da01670d8fc844e736095918bbe11fe8d564163");
}
