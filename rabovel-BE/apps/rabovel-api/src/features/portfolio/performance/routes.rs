use super::handlers::portfolio_performance_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route("/portfolio/performance", get(portfolio_performance_handler))
}
