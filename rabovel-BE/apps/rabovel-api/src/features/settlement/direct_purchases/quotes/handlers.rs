use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteRequest;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteResponse;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(post, path = "/investor/quote", request_body = QuoteRequest, responses((status = 200, body = QuoteResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn investor_quote_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<QuoteRequest>,
) -> Result<Json<QuoteResponse>, ApiError> {
    Ok(Json(
        super::usecases::investor_quote(&state, user, request).await?,
    ))
}
