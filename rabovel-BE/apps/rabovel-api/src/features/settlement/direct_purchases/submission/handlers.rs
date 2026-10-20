use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::settlement::direct_purchases::submission::dto::PurchaseSettlementResponse;
use crate::features::settlement::direct_purchases::submission::dto::SubmitPurchaseRequest;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(post, path = "/investor/purchase/submit", request_body = SubmitPurchaseRequest, responses((status = 200, body = PurchaseSettlementResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn submit_investor_purchase_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<SubmitPurchaseRequest>,
) -> Result<Json<PurchaseSettlementResponse>, ApiError> {
    Ok(Json(
        super::usecases::submit_investor_purchase(&state, user, request).await?,
    ))
}
