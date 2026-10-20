use std::sync::Arc;
pub(crate) trait ExecutionMarket {
    fn market_queries(&self) -> &Arc<dyn portfolio_trading::MarketQueries>;
}
