use crate::*;

#[async_trait::async_trait]
pub trait BalanceQueries: Send + Sync {
    async fn get_portfolio(&self, user_id: &str) -> Result<PortfolioSnapshot, TradingError>;
}
