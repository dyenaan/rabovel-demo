use super::handlers::create_asset_draft_handler;
use super::handlers::delete_asset_draft_handler;
use super::handlers::get_asset_draft_handler;
use super::handlers::list_asset_drafts_handler;
use super::handlers::update_asset_draft_handler;
use crate::app::state::GatewayState;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/issuer/assets", get(list_asset_drafts_handler))
        .route("/issuer/assets", post(create_asset_draft_handler))
        .route("/issuer/assets/:asset_id", get(get_asset_draft_handler))
        .route(
            "/issuer/assets/:asset_id",
            patch(update_asset_draft_handler),
        )
        .route(
            "/issuer/assets/:asset_id",
            delete(delete_asset_draft_handler),
        )
}
