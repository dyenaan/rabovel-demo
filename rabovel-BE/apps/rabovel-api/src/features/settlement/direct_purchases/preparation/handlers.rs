use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::settlement::direct_purchases::preparation::dto::PreparePurchaseRequest;
use crate::features::settlement::direct_purchases::preparation::dto::PreparedPurchaseResponse;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(post, path = "/investor/purchase/prepare", request_body = PreparePurchaseRequest, responses((status = 200, body = PreparedPurchaseResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn prepare_investor_purchase_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<PreparePurchaseRequest>,
) -> Result<Json<PreparedPurchaseResponse>, ApiError> {
    Ok(Json(
        super::usecases::prepare_investor_purchase(&state, user, request).await?,
    ))
}
