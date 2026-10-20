use super::handlers::submit_asset_backing_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route(
        "/issuer/assets/:asset_id/backing",
        post(submit_asset_backing_handler),
    )
}
