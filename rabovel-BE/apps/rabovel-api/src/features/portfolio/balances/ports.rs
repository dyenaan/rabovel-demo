use std::sync::Arc;
pub(crate) trait Balances {
    fn balance_queries(&self) -> &Arc<dyn portfolio_trading::BalanceQueries>;
}
