use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{
    cancel_order_handler, investor_catalog_handler, investor_quote_handler, list_orders_handler,
    place_order_handler, portfolio_handler, prepare_investor_purchase_handler,
    submit_investor_purchase_handler, GatewayState,
};

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/portfolio", get(portfolio_handler))
        .route("/investor/catalog", get(investor_catalog_handler))
        .route("/investor/quote", post(investor_quote_handler))
        .route(
            "/investor/purchase/prepare",
            post(prepare_investor_purchase_handler),
        )
        .route(
            "/investor/purchase/submit",
            post(submit_investor_purchase_handler),
        )
        .route(
            "/orders",
            post(place_order_handler).get(list_orders_handler),
        )
        .route("/orders/:order_id", delete(cancel_order_handler))
}
