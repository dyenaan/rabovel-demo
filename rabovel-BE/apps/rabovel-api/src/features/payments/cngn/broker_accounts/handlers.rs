use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::payments::cngn::models::PaymentAssetStatus;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(get, path = "/payment-assets/cngn", responses((status = 200, body = PaymentAssetStatus), (status = 401, body = ApiErrorResponse)))]
pub(crate) async fn cngn_payment_asset_handler(
    State(state): State<super::context::FeatureContext>,
    _user: AuthenticatedUser,
) -> Json<PaymentAssetStatus> {
    Json(super::usecases::cngn_payment_asset(&state, _user).await)
}

#[utoipa::path(post, path = "/payment-assets/cngn", responses((status = 200, body = PaymentAssetStatus), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn create_broker_cngn_account_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<PaymentAssetStatus>, ApiError> {
    Ok(Json(
        super::usecases::create_broker_cngn_account(&state, user).await?,
    ))
}
