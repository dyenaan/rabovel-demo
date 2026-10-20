use super::handlers::password_login_handler;
use super::handlers::password_register_handler;
use crate::app::state::GatewayState;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/auth/register", post(password_register_handler))
        .route("/auth/login", post(password_login_handler))
}
