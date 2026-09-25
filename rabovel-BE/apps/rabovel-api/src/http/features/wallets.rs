use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    authorize_wallet_handler, create_investor_cngn_account_handler, investor_cngn_account_handler,
    list_wallets_handler, solana_wallet_challenge_handler, wallet_challenge_handler,
    wallet_link_handler, GatewayState,
};

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/wallet/challenges", post(wallet_challenge_handler))
        .route(
            "/wallet/solana/challenges",
            post(solana_wallet_challenge_handler),
        )
        .route("/wallet/link", post(wallet_link_handler))
        .route("/wallet/authorize", post(authorize_wallet_handler))
        .route("/wallets", get(list_wallets_handler))
        .route(
            "/wallet/payment-assets/cngn",
            get(investor_cngn_account_handler).post(create_investor_cngn_account_handler),
        )
}
