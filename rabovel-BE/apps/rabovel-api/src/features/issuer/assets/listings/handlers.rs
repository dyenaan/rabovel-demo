use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::http::errors::ApiError;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(post, path = "/issuer/assets/{asset_id}/listing", responses((status = 200, body = AssetDraftResponse), (status = 403, body = ApiErrorResponse), (status = 409, body = ApiErrorResponse)))]
pub(crate) async fn publish_asset_listing_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    Ok(Json(
        super::usecases::publish_asset_listing(&state, user, asset_id).await?,
    ))
}
