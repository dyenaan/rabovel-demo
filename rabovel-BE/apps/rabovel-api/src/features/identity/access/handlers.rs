use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::access::dto::WalletAuthorizationRequest;
use crate::features::identity::access::dto::WalletAuthorizationResponse;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(
    post,
    path = "/wallet/authorize",
    request_body = WalletAuthorizationRequest,
    responses((status = 200, body = WalletAuthorizationResponse), (status = 403, body = ApiErrorResponse))
)]
pub(crate) async fn authorize_wallet_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<WalletAuthorizationRequest>,
) -> Result<Json<WalletAuthorizationResponse>, ApiError> {
    Ok(Json(
        super::usecases::authorize_wallet(&state, user, request).await?,
    ))
}
