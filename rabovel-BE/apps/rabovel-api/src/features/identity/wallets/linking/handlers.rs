use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::wallets::linking::dto::SolanaWalletChallengeRequest;
use crate::features::identity::wallets::linking::dto::WalletChallengeRequest;
use crate::features::identity::wallets::linking::dto::WalletChallengeResponse;
use crate::features::identity::wallets::linking::dto::WalletLinkRequest;
use crate::features::identity::wallets::linking::dto::WalletLinkResponse;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(
    post,
    path = "/wallet/challenges",
    request_body = WalletChallengeRequest,
    responses((status = 200, body = WalletChallengeResponse), (status = 401, body = ApiErrorResponse))
)]
pub(crate) async fn wallet_challenge_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<WalletChallengeRequest>,
) -> Result<Json<WalletChallengeResponse>, ApiError> {
    Ok(Json(
        super::usecases::wallet_challenge(&state, user, request).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/wallet/solana/challenges",
    request_body = SolanaWalletChallengeRequest,
    responses((status = 200, body = WalletChallengeResponse), (status = 401, body = ApiErrorResponse))
)]
pub(crate) async fn solana_wallet_challenge_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<SolanaWalletChallengeRequest>,
) -> Result<Json<WalletChallengeResponse>, ApiError> {
    Ok(Json(
        super::usecases::solana_wallet_challenge(&state, user, request).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/wallet/link",
    request_body = WalletLinkRequest,
    responses((status = 200, body = WalletLinkResponse), (status = 401, body = ApiErrorResponse))
)]
pub(crate) async fn wallet_link_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<WalletLinkRequest>,
) -> Result<Json<WalletLinkResponse>, ApiError> {
    Ok(Json(
        super::usecases::wallet_link(&state, user, request).await?,
    ))
}
