use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    auth_me_handler, google_authorization_handler, google_callback_handler, logout_handler,
    password_login_handler, password_register_handler, GatewayState,
};

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/auth/google/start", get(google_authorization_handler))
        .route(
            "/auth/google/callback",
            axum::routing::post(google_callback_handler),
        )
        .route("/auth/me", get(auth_me_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/auth/register", post(password_register_handler))
        .route("/auth/login", post(password_login_handler))
}
