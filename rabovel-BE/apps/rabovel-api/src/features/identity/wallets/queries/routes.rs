use super::handlers::list_wallets_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route("/wallets", get(list_wallets_handler))
}
