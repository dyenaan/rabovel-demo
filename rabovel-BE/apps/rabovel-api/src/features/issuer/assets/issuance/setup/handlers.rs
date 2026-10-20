use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::issuance::setup::dto::AssetSetupOperationResponse;
use crate::http::errors::ApiError;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(get, path = "/issuer/assets/{asset_id}/setup", responses((status = 200, body = AssetSetupOperationResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
pub(crate) async fn get_asset_setup_operation_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetSetupOperationResponse>, ApiError> {
    Ok(Json(
        super::usecases::get_asset_setup_operation(&state, user, asset_id).await?,
    ))
}

#[utoipa::path(post, path = "/issuer/assets/{asset_id}/setup", responses((status = 200, body = AssetSetupOperationResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
pub(crate) async fn start_asset_setup_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetSetupOperationResponse>, ApiError> {
    Ok(Json(
        super::usecases::start_asset_setup(&state, user, asset_id).await?,
    ))
}
