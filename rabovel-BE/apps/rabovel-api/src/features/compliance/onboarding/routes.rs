use super::handlers::accept_terms_handler;
use super::handlers::onboarding_status_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/onboarding/status", get(onboarding_status_handler))
        .route("/onboarding/terms", post(accept_terms_handler))
}
