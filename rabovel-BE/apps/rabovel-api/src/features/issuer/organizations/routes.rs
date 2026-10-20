use super::handlers::issuer_onboarding_handler;
use super::handlers::issuer_overview_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/issuer/overview", get(issuer_overview_handler))
        .route("/issuer/onboarding", post(issuer_onboarding_handler))
}
