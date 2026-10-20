use super::handlers::asset_nav_history_handler;
use super::handlers::market_price_history_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route(
            "/markets/:market_id/price-history",
            get(market_price_history_handler),
        )
        .route(
            "/assets/:asset_id/nav-history",
            get(asset_nav_history_handler),
        )
}
