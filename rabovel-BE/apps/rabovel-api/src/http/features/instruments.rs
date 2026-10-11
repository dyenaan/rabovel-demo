use axum::{routing::get, Router};

use crate::{
    get_asset_handler, get_market_handler, instruments_handler, list_assets_handler,
    list_markets_handler, market_order_book_handler, market_price_history_handler,
    market_trades_handler, asset_nav_history_handler, GatewayState,
};

/// Instruments, markets and assets are views of one catalog.
pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/instruments", get(instruments_handler))
        .route("/markets", get(list_markets_handler))
        .route("/markets/:market_id", get(get_market_handler))
        .route("/markets/:market_id/order-book", get(market_order_book_handler))
        .route(
            "/markets/:market_id/price-history",
            get(market_price_history_handler),
        )
        .route("/markets/:market_id/trades", get(market_trades_handler))
        .route("/assets", get(list_assets_handler))
        .route("/assets/:asset_id", get(get_asset_handler))
        .route("/assets/:asset_id/nav-history", get(asset_nav_history_handler))
}
