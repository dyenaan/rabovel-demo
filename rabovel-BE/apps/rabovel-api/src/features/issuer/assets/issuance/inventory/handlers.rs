use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::issuance::inventory::dto::InitialInventoryResponse;
use crate::http::errors::ApiError;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(get, path = "/issuer/assets/{asset_id}/inventory", responses((status = 200, body = InitialInventoryResponse), (status = 403, body = ApiErrorResponse), (status = 409, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn get_initial_inventory_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<InitialInventoryResponse>, ApiError> {
    Ok(Json(
        super::usecases::get_initial_inventory(&state, user, asset_id).await?,
    ))
}

#[utoipa::path(post, path = "/issuer/assets/{asset_id}/inventory", responses((status = 200, body = InitialInventoryResponse), (status = 403, body = ApiErrorResponse), (status = 409, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn issue_initial_inventory_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<InitialInventoryResponse>, ApiError> {
    Ok(Json(
        super::usecases::issue_initial_inventory(&state, user, asset_id).await?,
    ))
}
