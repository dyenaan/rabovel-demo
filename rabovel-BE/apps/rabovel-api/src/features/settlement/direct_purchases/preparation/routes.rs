use super::handlers::prepare_investor_purchase_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route(
        "/investor/purchase/prepare",
        post(prepare_investor_purchase_handler),
    )
}
