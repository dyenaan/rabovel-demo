use super::handlers::submit_kyc_case_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route("/kyc/cases", post(submit_kyc_case_handler))
}
