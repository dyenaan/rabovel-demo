use super::*;
use crate::now_secs;
use crate::ExecutedTrade;
use crate::TradingError;

#[async_trait::async_trait]
impl crate::trading::repricing::ports::Repricing for EngineTradingService {
    async fn reprice_resting(&self) -> Result<Vec<ExecutedTrade>, TradingError> {
        let decimals = self.config.price_decimals;
        let symbols = self.lock()?.engine.symbols_with_resting_orders();
        // Fetch prices without holding the lock across awaits.
        let mut prices = Vec::new();
        for symbol in symbols {
            let reference = self
                .prices
                .reference_price(&symbol, decimals)
                .await
                .map_err(TradingError::Unavailable)?;
            if let Some(reference) = reference.filter(|reference| {
                now_secs().saturating_sub(reference.as_of) <= self.config.max_price_age_secs
            }) {
                prices.push((symbol, reference.price_base_units));
            }
        }
        let now = now_secs();
        let mut state = self.lock()?;
        let mut trades = Vec::new();
        for (symbol, price) in prices {
            let fills = state.engine.reprice(&symbol, price);
            trades.extend(state.apply_fills(&fills, now, decimals));
        }
        Ok(trades)
    }
}
