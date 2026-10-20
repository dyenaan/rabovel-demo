use super::handlers::publish_asset_listing_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route(
        "/issuer/assets/:asset_id/listing",
        post(publish_asset_listing_handler),
    )
}
