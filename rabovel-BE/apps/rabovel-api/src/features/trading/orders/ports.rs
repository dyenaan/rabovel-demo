use std::sync::Arc;
pub(crate) trait Orders {
    fn order_queries(&self) -> &Arc<dyn portfolio_trading::OrderQueries>;
}
