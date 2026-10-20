use super::handlers::cancel_order;
use super::handlers::get_order;
use super::handlers::list_orders;
use super::handlers::place_order;
use crate::GatewayState;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/orders", post(place_order).get(list_orders))
        .route("/orders/:order_id", get(get_order).delete(cancel_order))
}
