use super::handlers::create_investor_cngn_account_handler;
use super::handlers::investor_cngn_account_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route(
            "/wallet/payment-assets/cngn",
            get(investor_cngn_account_handler),
        )
        .route(
            "/wallet/payment-assets/cngn",
            post(create_investor_cngn_account_handler),
        )
}
