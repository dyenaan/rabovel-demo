use crate::app::test_support::*;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use identity::UserRole;
use tower::ServiceExt;

#[tokio::test]
async fn approved_issuer_creates_and_reads_valid_asset_draft() {
    let (app, token) = login(test_app_with_role(UserRole::Issuer)).await;
    let payment_asset = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/payment-assets/cngn")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(payment_asset.status(), StatusCode::OK);
    let payment_asset = body_json(payment_asset).await;
    assert_eq!(payment_asset["network"], "testnet");
    assert_eq!(payment_asset["decimals"], 6);
    assert_eq!(payment_asset["ready"], true);
    let organization = serde_json::json!({
        "legal_name": "Demo Issuer Ltd", "organization_type": "company", "registration_number": "RC-123",
        "jurisdiction": "Nigeria", "registered_address": "1 Demo Street", "representative_name": "Ada Demo",
        "representative_title": "Director", "document_reference": "demo-registration.pdf",
        "beneficial_owners_confirmed": true, "information_certified": true
    });
    let onboarded = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/issuer/onboarding")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(organization.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(onboarded.status(), StatusCode::OK);

    let broker_account = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/payment-assets/cngn")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(broker_account.status(), StatusCode::OK);
    let broker_account = body_json(broker_account).await;
    assert_eq!(broker_account["broker_account_verified"], true);
    assert_eq!(broker_account["fee_payer_address"], "admin");

    let draft = serde_json::json!({
        "instrument_code": "RABO-NG-TELCO", "name": "Rabovel Nigeria Telco", "ticker": "RABOTELCO",
        "market": "NGX", "share_class": "Ordinary", "asset_type": "equity", "decimals": 0,
        "authorized_units": "1000000", "settlement_currency": "CNGN",
        "representation": "1 token = 1 simulated beneficial entitlement to 1 share",
        "rights_description": "Simulated ordinary-share entitlement for the Rabovel prototype.",
        "disclosure": issuer_assets::assets::drafts::models::DEMO_DISCLOSURE,
        "metadata": { "name": "Rabovel Nigeria Telco", "symbol": "RABOTELCO", "description": "Demo equity", "image_uri": null, "external_url": null, "metadata_uri": null, "additional_metadata": [] }
    });

    for (ticker, status) in [
        ("RABO-NG-TELCO", StatusCode::BAD_REQUEST),
        ("DANGCEM", StatusCode::CONFLICT),
    ] {
        let mut rejected = draft.clone();
        rejected["ticker"] = serde_json::json!(ticker);
        rejected["metadata"]["symbol"] = serde_json::json!(ticker);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/issuer/assets")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(rejected.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status, "{ticker}");
    }

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/issuer/assets")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(draft.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::OK);

    let created = body_json(created).await;
    assert_eq!(created["status"], "draft");
    assert_eq!(created["issued_units"], "0");
    assert!(created["mint_address"].is_null());
    let asset_id = created["asset_id"].as_str().unwrap();

    let mut revised = draft.clone();
    revised["name"] = serde_json::json!("Rabovel Nigeria Telco Revised");
    revised["metadata"]["name"] = serde_json::json!("Rabovel Nigeria Telco Revised");
    let updated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/issuer/assets/{asset_id}"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(revised.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(updated.status(), StatusCode::OK);
    assert_eq!(
        body_json(updated).await["draft"]["name"],
        "Rabovel Nigeria Telco Revised"
    );

    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/issuer/assets")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    assert_eq!(body_json(listed).await.as_array().unwrap().len(), 1);

    let duplicate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/issuer/assets")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(draft.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(duplicate.status(), StatusCode::BAD_REQUEST);

    let submitted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/issuer/assets/{asset_id}/submit"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submitted.status(), StatusCode::OK);
    let submitted = body_json(submitted).await;
    assert_eq!(submitted["status"], "approved_for_setup");
    assert_eq!(
        submitted["draft"]["metadata"]["metadata_uri"],
        format!("https://metadata.example/assets/{asset_id}/metadata.json")
    );

    let setup = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/issuer/assets/{asset_id}/setup"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(setup.status(), StatusCode::OK);
    let setup = body_json(setup).await;
    assert_eq!(setup["status"], "confirmed");
    assert_eq!(setup["stages"].as_array().unwrap().len(), 4);
    let operation_id = setup["operation_id"].as_str().unwrap();
    assert_eq!(setup["mint_address"], format!("mint-{operation_id}"));

    let replay = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/issuer/assets/{asset_id}/setup"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(body_json(replay).await["operation_id"], operation_id);

    let minted = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/issuer/assets/{asset_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let minted = body_json(minted).await;
    assert_eq!(minted["status"], "minted");
    assert_eq!(minted["mint_address"], format!("mint-{operation_id}"));

    let inventory = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/issuer/assets/{asset_id}/inventory"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(inventory.status(), StatusCode::OK);
    let inventory = body_json(inventory).await;
    assert_eq!(inventory["issuance_complete"], true);
    assert_eq!(inventory["wallet_allowlisted"], true);
    assert_eq!(inventory["inventory_balance"], "1000000");

    let backing = serde_json::json!({
        "summary": "Each token represents one simulated ordinary share held by the issuer.",
        "document_name": "share-register.pdf",
        "content_type": "application/pdf",
        "size_bytes": 2048
    });
    let backed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/issuer/assets/{asset_id}/backing"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(backing.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backed.status(), StatusCode::OK);
    let backed = body_json(backed).await;
    assert_eq!(
        backed["draft"]["backing"]["verification_status"],
        "verified"
    );
    assert_eq!(backed["draft"]["listing_status"], "not_listed");

    let published = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/issuer/assets/{asset_id}/listing"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::OK);
    assert_eq!(
        body_json(published).await["draft"]["listing_status"],
        "live"
    );

    // A live listing joins the same market catalog as the NGX instruments.
    let market = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/markets/mkt_ngx_rabotelco")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(market.status(), StatusCode::OK);
    let market = body_json(market).await;
    assert_eq!(market["data"]["symbol"], "RABOTELCO/NGN");
    assert_eq!(market["data"]["origin"], "issuer");
    assert_eq!(market["data"]["assetId"], asset_id);
    assert_eq!(
        market["data"]["mintAddress"],
        format!("mint-{operation_id}")
    );

    let edit_after_approval = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/issuer/assets/{asset_id}"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(revised.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(edit_after_approval.status(), StatusCode::FORBIDDEN);
}
