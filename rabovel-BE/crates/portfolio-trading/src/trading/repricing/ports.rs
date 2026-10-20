use crate::*;

#[async_trait::async_trait]
pub trait Repricing: Send + Sync {
    /// Lets the house fill resting limit orders that the current reference
    /// prices now satisfy. Called on a timer so limits fill as the market
    /// moves, not only when someone else trades the symbol. Returns the
    /// trades for event publication.
    async fn reprice_resting(&self) -> Result<Vec<ExecutedTrade>, TradingError> {
        Ok(Vec::new())
    }
}
