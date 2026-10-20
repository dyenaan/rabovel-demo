use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::payments::cngn::models::WalletPaymentAssetStatus;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(get, path = "/wallet/payment-assets/cngn", responses((status = 200, body = WalletPaymentAssetStatus), (status = 401, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn investor_cngn_account_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<WalletPaymentAssetStatus>, ApiError> {
    Ok(Json(
        super::usecases::investor_cngn_account(&state, user).await?,
    ))
}

#[utoipa::path(post, path = "/wallet/payment-assets/cngn", responses((status = 200, body = WalletPaymentAssetStatus), (status = 401, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn create_investor_cngn_account_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<WalletPaymentAssetStatus>, ApiError> {
    Ok(Json(
        super::usecases::create_investor_cngn_account(&state, user).await?,
    ))
}
