use crate::features::catalog::instruments::InstrumentProvider;
use crate::features::market_data::prices::ports::PriceFeed;
use async_trait::async_trait;
use portfolio_trading::ReferencePrice;
use portfolio_trading::ReferencePriceSource;
use std::sync::Arc;

/// Reference prices for the matching engine: only instruments in the catalog
/// (NGX-listed or live issuer listings) are tradable, and each is priced by
/// the configured feed.
pub struct CatalogReferencePrices {
    instruments: Arc<dyn InstrumentProvider>,
    feed: Arc<dyn PriceFeed>,
}
impl CatalogReferencePrices {
    pub fn new(instruments: Arc<dyn InstrumentProvider>, feed: Arc<dyn PriceFeed>) -> Self {
        Self { instruments, feed }
    }
}
#[async_trait]
impl ReferencePriceSource for CatalogReferencePrices {
    async fn reference_price(
        &self,
        symbol: &str,
        price_decimals: u8,
    ) -> Result<Option<ReferencePrice>, String> {
        let Some(instrument) = self.instruments.find(symbol).await? else {
            return Ok(None);
        };
        let price = self.feed.latest(&instrument.ticker, price_decimals)?;
        Ok(Some(ReferencePrice {
            price_base_units: price.price_base_units,
            as_of: price.as_of,
        }))
    }
}
