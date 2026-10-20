use std::sync::Arc;
pub(crate) trait ReferencePrices {
    fn price_feed(&self) -> &Arc<dyn crate::features::market_data::prices::ports::PriceFeed>;
}
pub(crate) trait PriceDecimals {
    fn price_decimals(&self) -> u8;
}

use super::models::MarketPrice;

use std::time::SystemTime;

use std::time::UNIX_EPOCH;

pub trait PriceFeed: Send + Sync {
    /// The price `ticker` had (or has) at unix second `at`.
    fn price_at(&self, ticker: &str, payment_decimals: u8, at: u64) -> Result<MarketPrice, String>;

    fn latest(&self, ticker: &str, payment_decimals: u8) -> Result<MarketPrice, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.price_at(ticker, payment_decimals, now)
    }
}
