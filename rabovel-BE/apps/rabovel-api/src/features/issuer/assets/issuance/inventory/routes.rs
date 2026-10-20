use super::handlers::get_initial_inventory_handler;
use super::handlers::issue_initial_inventory_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route(
            "/issuer/assets/:asset_id/inventory",
            get(get_initial_inventory_handler),
        )
        .route(
            "/issuer/assets/:asset_id/inventory",
            post(issue_initial_inventory_handler),
        )
}
