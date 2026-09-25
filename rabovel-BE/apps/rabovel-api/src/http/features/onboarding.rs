use axum::{
    routing::{get, post},
    Router,
};

use crate::{accept_terms_handler, onboarding_status_handler, GatewayState};

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/onboarding/status", get(onboarding_status_handler))
        .route("/onboarding/terms", post(accept_terms_handler))
}
