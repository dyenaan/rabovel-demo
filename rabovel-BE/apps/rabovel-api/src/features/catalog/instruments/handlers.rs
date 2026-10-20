use crate::features::catalog::instruments::InstrumentCatalog;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(get, path = "/instruments", responses((status = 200, body = InstrumentCatalog), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn instruments_handler(
    State(state): State<super::context::FeatureContext>,
) -> Result<Json<InstrumentCatalog>, ApiError> {
    Ok(Json(super::usecases::instruments(&state).await?))
}
