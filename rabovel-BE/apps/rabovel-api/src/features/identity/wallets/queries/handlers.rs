use crate::features::identity::access::actor::AuthenticatedUser;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;
use identity::WalletConnection;

#[utoipa::path(get, path = "/wallets", responses((status = 200, body = [WalletConnection]), (status = 401, body = ApiErrorResponse)))]
pub(crate) async fn list_wallets_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<WalletConnection>>, ApiError> {
    Ok(Json(super::usecases::list_wallets(&state, user).await?))
}
