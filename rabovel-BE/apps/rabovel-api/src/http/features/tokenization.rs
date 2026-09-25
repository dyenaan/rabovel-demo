use axum::{
    routing::{get, post},
    Router,
};

use crate::{tokenization_request_handler, tokenization_status_handler, GatewayState};

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/tokenization/requests", post(tokenization_request_handler))
        .route(
            "/tokenization/requests/:request_id",
            get(tokenization_status_handler),
        )
}
