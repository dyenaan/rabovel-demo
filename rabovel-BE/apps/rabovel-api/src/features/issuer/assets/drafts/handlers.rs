use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::features::issuer::assets::drafts::dto::CreateAssetDraftRequest;
use crate::http::errors::ApiError;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;

#[utoipa::path(post, path = "/issuer/assets", request_body = CreateAssetDraftRequest, responses((status = 200, body = AssetDraftResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse)))]
pub(crate) async fn create_asset_draft_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<CreateAssetDraftRequest>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    Ok(Json(
        super::usecases::create_asset_draft(&state, user, request).await?,
    ))
}

#[utoipa::path(get, path = "/issuer/assets", responses((status = 200, body = [AssetDraftResponse]), (status = 403, body = ApiErrorResponse)))]
pub(crate) async fn list_asset_drafts_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<AssetDraftResponse>>, ApiError> {
    Ok(Json(
        super::usecases::list_asset_drafts(&state, user).await?,
    ))
}

#[utoipa::path(get, path = "/issuer/assets/{asset_id}", responses((status = 200, body = AssetDraftResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
pub(crate) async fn get_asset_draft_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    Ok(Json(
        super::usecases::get_asset_draft(&state, user, asset_id).await?,
    ))
}

#[utoipa::path(patch, path = "/issuer/assets/{asset_id}", request_body = CreateAssetDraftRequest, responses((status = 200, body = AssetDraftResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
pub(crate) async fn update_asset_draft_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
    Json(request): Json<CreateAssetDraftRequest>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    Ok(Json(
        super::usecases::update_asset_draft(&state, user, asset_id, request).await?,
    ))
}

#[utoipa::path(delete, path = "/issuer/assets/{asset_id}", responses((status = 204), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
pub(crate) async fn delete_asset_draft_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    super::usecases::delete_asset_draft(&state, user, asset_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
