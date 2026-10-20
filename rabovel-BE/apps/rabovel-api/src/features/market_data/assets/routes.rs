use super::handlers::get_asset_handler;
use super::handlers::list_assets_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/assets", get(list_assets_handler))
        .route("/assets/:asset_id", get(get_asset_handler))
}
