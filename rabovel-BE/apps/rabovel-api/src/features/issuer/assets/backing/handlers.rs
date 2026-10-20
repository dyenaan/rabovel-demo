use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::backing::dto::SubmitBackingRequest;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::http::errors::ApiError;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(post, path = "/issuer/assets/{asset_id}/backing", request_body = SubmitBackingRequest, responses((status = 200, body = AssetDraftResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse)))]
pub(crate) async fn submit_asset_backing_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
    Json(request): Json<SubmitBackingRequest>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    Ok(Json(
        super::usecases::submit_asset_backing(&state, user, asset_id, request).await?,
    ))
}
