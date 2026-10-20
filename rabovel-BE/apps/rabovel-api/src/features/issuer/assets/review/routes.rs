use super::handlers::submit_asset_for_demo_review_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route(
        "/issuer/assets/:asset_id/submit",
        post(submit_asset_for_demo_review_handler),
    )
}
