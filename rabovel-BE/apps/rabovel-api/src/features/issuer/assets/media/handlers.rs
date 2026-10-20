use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::features::issuer::assets::media::dto::ConfirmImageUploadRequest;
use crate::features::issuer::assets::media::dto::ImageUploadRequest;
use crate::features::issuer::assets::media::dto::ImageUploadTicket;
use crate::http::errors::ApiError;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Json;

pub(crate) async fn create_asset_image_upload_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
    Json(request): Json<ImageUploadRequest>,
) -> Result<Json<ImageUploadTicket>, ApiError> {
    Ok(Json(
        super::usecases::create_asset_image_upload(&state, user, asset_id, request).await?,
    ))
}

pub(crate) async fn confirm_asset_image_upload_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
    Json(request): Json<ConfirmImageUploadRequest>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    Ok(Json(
        super::usecases::confirm_asset_image_upload(&state, user, asset_id, request).await?,
    ))
}
