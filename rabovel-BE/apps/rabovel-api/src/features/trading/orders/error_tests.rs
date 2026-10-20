use crate::app::test_support::body_json;
use crate::features::trading::orders::providers::events::MessagingTradingEvents;
use crate::router_with_state;
use crate::GatewayStateBuilder;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use identity::AuthError;
use portfolio_trading::OrderAccessPolicy;
use portfolio_trading::OrderAction;
use portfolio_trading::OrderActor;
use portfolio_trading::OrderUseCases;
use portfolio_trading::StubPortfolioTradingService;
use std::sync::Arc;
use tower::ServiceExt;

struct RejectAccess(AuthError);

struct AllowAccess;

#[async_trait]
impl OrderAccessPolicy for AllowAccess {
    type Error = AuthError;

    async fn authorize(&self, _: &OrderActor, _: OrderAction) -> Result<(), AuthError> {
        Ok(())
    }
}

#[async_trait]
impl OrderAccessPolicy for RejectAccess {
    type Error = AuthError;

    async fn authorize(&self, _: &OrderActor, _: OrderAction) -> Result<(), AuthError> {
        Err(self.0.clone())
    }
}

#[tokio::test]
async fn typed_policy_rejections_keep_http_status_codes_and_messages() {
    for (error, status, code) in [
        (
            AuthError::Unauthorized,
            StatusCode::UNAUTHORIZED,
            "unauthorized",
        ),
        (
            AuthError::MissingWallet,
            StatusCode::NOT_FOUND,
            "missing_wallet",
        ),
        (
            AuthError::WalletNotOwned,
            StatusCode::FORBIDDEN,
            "wallet_not_owned",
        ),
        (
            AuthError::MfaRequired,
            StatusCode::FORBIDDEN,
            "mfa_required",
        ),
        (
            AuthError::OnboardingIncomplete,
            StatusCode::FORBIDDEN,
            "onboarding_incomplete",
        ),
        (
            AuthError::RateLimited,
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
        ),
        (
            AuthError::ServiceUnavailable,
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
        ),
    ] {
        let mut state = GatewayStateBuilder::default().build();
        let token =
            crate::features::identity::sessions::usecases::issue_session(&state, "alice", Some(42))
                .await
                .unwrap();
        state.order_use_cases = Arc::new(OrderUseCases::new(
            Arc::new(StubPortfolioTradingService::default()),
            Arc::new(RejectAccess(error.clone())),
            Arc::new(MessagingTradingEvents {
                producer: state.event_producer.clone(),
            }),
        ));
        let app = router_with_state(state);
        for (method, uri, body) in [
            (
                "POST",
                "/orders",
                r#"{"symbol":"dangcem","side":"buy","order_type":"market","quantity":"1","limit_price":null}"#,
            ),
            ("GET", "/orders", ""),
            ("GET", "/orders/ord_1", ""),
            ("DELETE", "/orders/ord_1", ""),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(uri)
                        .header("authorization", format!("Bearer {token}"))
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), status, "{method} {uri}");
            let response = body_json(response).await;
            assert_eq!(response["code"], code);
            let message = if status.is_server_error() {
                "The service is temporarily unavailable.".to_string()
            } else {
                error.to_string()
            };
            assert_eq!(response["message"], message);
        }
    }
}

#[tokio::test]
async fn execution_failures_keep_existing_http_error_contract() {
    let mut state = GatewayStateBuilder::default().build();
    let token = crate::features::identity::sessions::usecases::issue_session(&state, "alice", None)
        .await
        .unwrap();
    state.order_use_cases = Arc::new(OrderUseCases::new(
        Arc::new(StubPortfolioTradingService::default()),
        Arc::new(AllowAccess),
        Arc::new(MessagingTradingEvents {
            producer: state.event_producer.clone(),
        }),
    ));
    let app = router_with_state(state);
    for (method, uri, body, status, message) in [
        (
            "POST",
            "/orders",
            r#"{"symbol":"dangcem","side":"buy","order_type":"market","quantity":"0","limit_price":null}"#,
            StatusCode::BAD_REQUEST,
            "invalid quantity: 0",
        ),
        (
            "GET",
            "/orders/missing",
            "",
            StatusCode::NOT_FOUND,
            "order not found",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        let response = body_json(response).await;
        assert_eq!(response["code"], "policy_violation");
        assert_eq!(response["message"], message);
    }
}
