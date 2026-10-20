use super::handlers::kyc_webhook_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route("/kyc/webhook", post(kyc_webhook_handler))
}
