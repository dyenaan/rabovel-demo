use axum::{routing::get, Router};

use crate::{instruments_handler, GatewayState};

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route("/instruments", get(instruments_handler))
}
