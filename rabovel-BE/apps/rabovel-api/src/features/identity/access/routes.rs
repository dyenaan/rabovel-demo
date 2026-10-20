use super::handlers::authorize_wallet_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route("/wallet/authorize", post(authorize_wallet_handler))
}
