use super::handlers::get_asset_setup_operation_handler;
use super::handlers::start_asset_setup_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route(
            "/issuer/assets/:asset_id/setup",
            get(get_asset_setup_operation_handler),
        )
        .route(
            "/issuer/assets/:asset_id/setup",
            post(start_asset_setup_handler),
        )
}
