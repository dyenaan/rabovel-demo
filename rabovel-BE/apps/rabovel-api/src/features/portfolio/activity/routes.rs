use super::handlers::portfolio_activity_handler;
use crate::app::state::GatewayState;
use axum::routing::get;
use axum::Router;

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new().route("/portfolio/activity", get(portfolio_activity_handler))
}
