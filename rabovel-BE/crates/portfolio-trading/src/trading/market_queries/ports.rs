use crate::*;

#[async_trait::async_trait]
pub trait MarketQueries: Send + Sync {
    /// Aggregated public book for `symbol`, best price first: resting client
    /// orders plus the house quote when its reference price is fresh.
    async fn order_book(&self, symbol: &str, levels: usize) -> Result<OrderBook, TradingError> {
        let _ = levels;
        Ok(OrderBook {
            symbol: symbol.to_ascii_uppercase(),
            bids: Vec::new(),
            asks: Vec::new(),
        })
    }
    /// Most recent matched trades for `symbol`, newest first. Anonymous: no
    /// order ids or counterparties.
    async fn public_trades(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<PublicTrade>, TradingError> {
        let _ = (symbol, limit);
        Ok(Vec::new())
    }
}
