use super::handlers::confirm_asset_image_upload_handler;
use super::handlers::create_asset_image_upload_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route(
            "/issuer/assets/:asset_id/image-upload",
            post(create_asset_image_upload_handler),
        )
        .route(
            "/issuer/assets/:asset_id/image-upload/confirm",
            post(confirm_asset_image_upload_handler),
        )
}
