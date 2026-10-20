use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    order_queries: Arc<dyn portfolio_trading::OrderQueries>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            order_queries: state.order_queries.clone(),
        }
    }
}
impl crate::features::trading::orders::ports::Orders for FeatureContext {
    fn order_queries(&self) -> &Arc<dyn portfolio_trading::OrderQueries> {
        &self.order_queries
    }
}
