use super::handlers::portfolio_holdings_handler;
use super::handlers::portfolio_summary_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/portfolio/summary", get(portfolio_summary_handler))
        .route("/portfolio/holdings", get(portfolio_holdings_handler))
}
