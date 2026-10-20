use super::handlers::auth_me_handler;
use super::handlers::logout_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/auth/me", get(auth_me_handler))
        .route("/auth/logout", post(logout_handler))
}
