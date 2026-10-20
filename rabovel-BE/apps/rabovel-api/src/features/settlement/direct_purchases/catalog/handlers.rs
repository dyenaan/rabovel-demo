use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::settlement::direct_purchases::catalog::dto::InvestorCatalogResponse;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(get, path = "/investor/catalog", responses((status = 200, body = InvestorCatalogResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn investor_catalog_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<InvestorCatalogResponse>, ApiError> {
    Ok(Json(super::usecases::investor_catalog(&state, user).await?))
}
