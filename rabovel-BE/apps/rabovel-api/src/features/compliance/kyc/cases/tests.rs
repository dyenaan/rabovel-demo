use crate::app::test_support::*;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn kyc_case_lifecycle_flips_onboarding_from_requires_review_toward_approved() {
    let (app, token) = login(test_app()).await;
    // Link a wallet and accept terms first so KYC approval is the last
    // gate standing between this profile and `Approved`.
    let challenge = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/challenges")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "address": "0x6da01670d8fc844e736095918bbe11fe8d564163",
                        "chain_id": 1,
                        "provider": "meta_mask"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let challenge = body_json(challenge).await;
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wallet/link")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "challenge_id": challenge["challenge_id"],
                        "signature": "valid-test-signature"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/terms")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"version": "2026-01"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let submit = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/kyc/cases")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "full_name": "Ada Lovelace",
                        "date_of_birth": "1990-01-01",
                        "country": "NG",
                        "document_type": "passport",
                        "document_reference": "vendor-ref-1"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit.status(), StatusCode::OK);
    let submit = body_json(submit).await;
    let case_id = submit["case_id"].as_str().unwrap().to_string();

    let advance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/kyc/_sandbox/advance")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"case_id": case_id}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(advance.status(), StatusCode::OK);

    let status = app
        .oneshot(
            Request::builder()
                .uri("/onboarding/status")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(status.status(), StatusCode::OK);
    let status = body_json(status).await;
    // MFA is still not enrolled in this test's profile, so full
    // `Approved` isn't reached, but KYC itself must now read verified.
    assert_eq!(status["profile"]["kyc_verified"], true);
}
