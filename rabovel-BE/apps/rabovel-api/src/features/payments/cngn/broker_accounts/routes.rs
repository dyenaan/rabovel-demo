use super::handlers::cngn_payment_asset_handler;
use super::handlers::create_broker_cngn_account_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/payment-assets/cngn", get(cngn_payment_asset_handler))
        .route(
            "/payment-assets/cngn",
            post(create_broker_cngn_account_handler),
        )
}
