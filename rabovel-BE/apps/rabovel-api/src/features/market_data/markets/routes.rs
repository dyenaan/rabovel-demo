use super::handlers::get_market_handler;
use super::handlers::list_markets_handler;
use super::handlers::market_order_book_handler;
use super::handlers::market_trades_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/markets", get(list_markets_handler))
        .route("/markets/:market_id", get(get_market_handler))
        .route(
            "/markets/:market_id/order-book",
            get(market_order_book_handler),
        )
        .route("/markets/:market_id/trades", get(market_trades_handler))
}
