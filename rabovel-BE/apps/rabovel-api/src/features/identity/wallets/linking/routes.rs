use super::handlers::solana_wallet_challenge_handler;
use super::handlers::wallet_challenge_handler;
use super::handlers::wallet_link_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/wallet/challenges", post(wallet_challenge_handler))
        .route(
            "/wallet/solana/challenges",
            post(solana_wallet_challenge_handler),
        )
        .route("/wallet/link", post(wallet_link_handler))
}
