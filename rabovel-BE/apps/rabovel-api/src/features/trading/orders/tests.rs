use crate::app::test_support::body_json;
use crate::app::test_support::link_phantom_wallet;
use crate::app::test_support::login;
use crate::app::test_support::test_security_policy;
use crate::app::test_support::FakePaymentAssets;
use crate::app::test_support::FakeWalletProof;
use crate::router_with_state;
use crate::GatewayStateBuilder;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use identity::UserRole;
use infrastructure::messaging;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::drafts::models::BackingEvidence;
use issuer_assets::assets::drafts::models::NewAssetDraft;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn orders_execute_against_the_matching_engine_and_publish_trades() {
    let producer = Arc::new(messaging::InMemoryEventProducer::default());
    let repository = crate::app::test_support::fixture_repository(UserRole::Trader);
    let listed = AssetDraftRecord {
        asset_id: "ast_rabotelco".into(),
        issuer_user_id: "usr_issuer".into(),
        canonical_key: "NGX:RABOTELCO:ORDINARY".into(),
        draft: NewAssetDraft {
            instrument_code: "RABO-NG-TELCO".into(),
            name: "Rabovel Nigeria Telco".into(),
            ticker: "RABOTELCO".into(),
            market: "NGX".into(),
            share_class: "Ordinary".into(),
            asset_type: "equity".into(),
            decimals: 0,
            authorized_units: 1_000_000,
            settlement_currency: "CNGN".into(),
            representation: "1 token = 1 simulated share".into(),
            rights_description: "Simulated".into(),
            disclosure: issuer_assets::assets::drafts::models::DEMO_DISCLOSURE.into(),
            metadata: issuer_assets::assets::drafts::models::AssetMetadata {
                name: "Rabovel Nigeria Telco".into(),
                symbol: "RABOTELCO".into(),
                description: "Demo equity".into(),
                image_uri: None,
                external_url: None,
                metadata_uri: None,
                additional_metadata: vec![],
            },
            backing: Some(BackingEvidence {
                summary: "Simulated share register".into(),
                document_name: "register.pdf".into(),
                content_type: "application/pdf".into(),
                size_bytes: 1,
                verification_status: "verified".into(),
                verified_at: 1,
            }),
            listing_status: "not_listed".into(),
        },
        status: "minted".into(),
        created_at: 1,
        updated_at: 1,
        mint_address: Some("mint-rabotelco".into()),
    };
    repository.assets.create_asset_draft(listed).await.unwrap();
    let app = router_with_state(
        GatewayStateBuilder::default()
            .persistence(repository.clone())
            .wallet_proof(Arc::new(FakeWalletProof))
            .security_policy(test_security_policy())
            .payment_assets(Arc::new(FakePaymentAssets))
            .event_producer(producer.clone())
            .build(),
    );
    let (app, token) = login(app).await;
    link_phantom_wallet(&app, &token).await;

    let send = |method: &str, uri: String, body: Option<serde_json::Value>| {
        let builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json");
        let request = builder
            .body(body.map_or_else(Body::empty, |body| Body::from(body.to_string())))
            .unwrap();
        app.clone().oneshot(request)
    };

    let placed = send(
        "POST",
        "/orders".into(),
        Some(serde_json::json!({
            "symbol": "dangcem",
            "side": "buy",
            "order_type": "market",
            "quantity": "3",
            "limit_price": null
        })),
    )
    .await
    .unwrap();
    assert_eq!(placed.status(), StatusCode::OK);
    let placed = body_json(placed).await;
    assert_eq!(placed["state"], "filled");
    assert_eq!(placed["filled_quantity"], "3");
    assert!(placed.get("trades").is_none());
    assert!(placed.get("user_id").is_none());
    assert_eq!(placed["fills"][0]["liquidity"], "principal");
    // The response never reveals counterparties.
    assert!(placed["fills"][0].get("seller").is_none());
    let order_id = placed["order_id"].as_str().unwrap().to_string();

    let order = send("GET", format!("/orders/{order_id}"), None)
        .await
        .unwrap();
    assert_eq!(order.status(), StatusCode::OK);
    assert_eq!(body_json(order).await["state"], "filled");
    let orders = body_json(send("GET", "/orders".into(), None).await.unwrap()).await;
    assert_eq!(orders.as_array().unwrap().len(), 1);
    let portfolio = body_json(send("GET", "/portfolio".into(), None).await.unwrap()).await;
    assert!(portfolio["balances"]
        .as_array()
        .unwrap()
        .iter()
        .any(|balance| balance["asset"] == "DANGCEM" && balance["total"] == "3"));

    let unknown = send(
        "POST",
        "/orders".into(),
        Some(serde_json::json!({
            "symbol": "NOT-LISTED",
            "side": "buy",
            "order_type": "market",
            "quantity": "1",
            "limit_price": null
        })),
    )
    .await
    .unwrap();
    assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);

    // An issuer asset is tradable only once it is listed live, and then
    // with no extra registration: same catalog, same engine.
    let issuer_order = serde_json::json!({
        "symbol": "rabotelco",
        "side": "buy",
        "order_type": "market",
        "quantity": "2",
        "limit_price": null
    });
    let unlisted = send("POST", "/orders".into(), Some(issuer_order.clone()))
        .await
        .unwrap();
    assert_eq!(unlisted.status(), StatusCode::BAD_REQUEST);
    repository
        .assets
        .publish_asset_listing("ast_rabotelco", "usr_issuer", 2)
        .await
        .unwrap();
    let markets = body_json(send("GET", "/markets".into(), None).await.unwrap()).await;
    let markets = markets["data"].as_array().unwrap();
    assert_eq!(markets.len(), 147);
    assert!(markets
        .iter()
        .any(|market| market["marketId"] == "mkt_ngx_dangcem" && market["origin"] == "ngx_listed"));
    assert!(markets
        .iter()
        .any(|market| market["marketId"] == "mkt_ngx_rabotelco" && market["origin"] == "issuer"));
    let filled = send("POST", "/orders".into(), Some(issuer_order))
        .await
        .unwrap();
    assert_eq!(filled.status(), StatusCode::OK);
    let filled = body_json(filled).await;
    assert_eq!(filled["state"], "filled");
    assert_eq!(filled["filled_quantity"], "2");

    let get = |uri: &str| {
        let uri = uri.to_string();
        let send = &send;
        async move {
            let response = send("GET", uri.clone(), None).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            body_json(response).await["data"].clone()
        }
    };

    // Portfolio views are derived from the same engine fills.
    let holdings = get("/portfolio/holdings").await;
    let dangcem = holdings
        .as_array()
        .unwrap()
        .iter()
        .find(|holding| holding["symbol"] == "DANGCEM")
        .unwrap()
        .clone();
    assert_eq!(dangcem["assetId"], "ast_ngx_dangcem");
    assert_eq!(dangcem["quantity"], "3");
    assert_eq!(holdings.as_array().unwrap().len(), 2);
    let summary = get("/portfolio/summary").await;
    // The fake wallet has 25 cNGN; count it once alongside engine positions.
    assert_eq!(summary["cashBalance"], "25.00");
    assert_ne!(summary["investedValue"], "0.00");
    let activity = get("/portfolio/activity").await;
    assert_eq!(activity[0]["type"], "ORDER_FILLED");
    assert_eq!(
        get("/portfolio/performance?range=1M")
            .await
            .as_array()
            .unwrap()
            .len(),
        30
    );

    // Market data comes from the engine: tape, book (house quote) and candles.
    let tape = get("/markets/mkt_ngx_dangcem/trades").await;
    assert_eq!(tape[0]["quantity"], "3");
    assert_eq!(tape[0]["side"], "BUY");
    let book = get("/markets/mkt_ngx_rabotelco/order-book").await;
    assert!(!book["asks"].as_array().unwrap().is_empty());
    assert!(!book["bids"].as_array().unwrap().is_empty());
    let candles = get("/markets/mkt_ngx_dangcem/price-history?range=1D").await;
    assert_eq!(candles.as_array().unwrap().len(), 24);
    assert_eq!(candles[23]["volume"], "3");

    // Assets are the same catalog as markets and instruments.
    let assets = get("/assets").await;
    assert_eq!(assets.as_array().unwrap().len(), 147);
    let issuer_asset = get("/assets/ast_rabotelco").await;
    assert_eq!(issuer_asset["symbol"], "RABOTELCO");
    assert_eq!(issuer_asset["origin"], "issuer");
    assert_eq!(issuer_asset["contractAddress"], "mint-rabotelco");
    assert_eq!(get("/assets/ast_ngx_gtco").await["origin"], "ngx_listed");
    assert_eq!(
        get("/assets/ast_ngx_gtco/nav-history")
            .await
            .as_array()
            .unwrap()
            .len(),
        90
    );

    let trading_events: Vec<(String, serde_json::Value)> = producer
        .published()
        .await
        .into_iter()
        .filter(|(topic, _, _)| {
            topic == events::topics::TRADING_ORDER_ACCEPTED_V1
                || topic == events::topics::TRADING_TRADE_EXECUTED_V1
        })
        .map(|(topic, _, payload)| (topic, serde_json::from_slice(&payload).unwrap()))
        .collect();
    assert_eq!(trading_events.len(), 4);
    assert_eq!(
        trading_events[0].0,
        events::topics::TRADING_ORDER_ACCEPTED_V1
    );
    assert_eq!(
        trading_events[1].0,
        events::topics::TRADING_TRADE_EXECUTED_V1
    );
    assert_eq!(
        trading_events[2].0,
        events::topics::TRADING_ORDER_ACCEPTED_V1
    );
    assert_eq!(
        trading_events[3].0,
        events::topics::TRADING_TRADE_EXECUTED_V1
    );
    let accepted = &trading_events[0].1;
    assert_eq!(accepted["producer"], "order-gateway");
    assert_eq!(accepted["payload"]["order_id"], order_id);
    assert_eq!(accepted["payload"]["symbol"], "dangcem");
    assert_eq!(accepted["payload"]["quantity"], "3");
    assert_eq!(trading_events[1].1["producer"], "matching-engine");
    let trades: Vec<&serde_json::Value> = trading_events
        .iter()
        .filter(|(topic, _)| topic == events::topics::TRADING_TRADE_EXECUTED_V1)
        .map(|(_, payload)| payload)
        .collect();
    assert_eq!(trades.len(), 2);
    assert_eq!(trades[1]["payload"]["symbol"], "RABOTELCO");
    let trade = &trades[0]["payload"];
    assert_eq!(trade["symbol"], "DANGCEM");
    assert_eq!(trade["quantity"], "3");
    assert_eq!(trade["seller"]["type"], "house");
    assert_eq!(trade["buyer"]["order_id"], order_id.as_str());
    assert_eq!(trade["house_inventory"]["type"], "house_delivers");
    assert_eq!(trade["house_inventory"]["minted"], 3);
}
