use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    house_half_spread_bps: u32,
    instrument_provider: Arc<dyn crate::features::catalog::instruments::ports::InstrumentProvider>,
    market_queries: Arc<dyn portfolio_trading::MarketQueries>,
    price_decimals: u8,
    price_feed: Arc<dyn crate::features::market_data::prices::ports::PriceFeed>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            house_half_spread_bps: state.house_half_spread_bps,
            instrument_provider: state.instrument_provider.clone(),
            market_queries: state.market_queries.clone(),
            price_decimals: state.price_decimals,
            price_feed: state.price_feed.clone(),
        }
    }
}
impl crate::features::trading::ports::HouseSpread for FeatureContext {
    fn house_half_spread_bps(&self) -> u32 {
        self.house_half_spread_bps
    }
}
impl crate::features::catalog::instruments::ports::CatalogSource for FeatureContext {
    fn instrument_provider(
        &self,
    ) -> &Arc<dyn crate::features::catalog::instruments::ports::InstrumentProvider> {
        &self.instrument_provider
    }
}
impl crate::features::market_data::markets::ports::ExecutionMarket for FeatureContext {
    fn market_queries(&self) -> &Arc<dyn portfolio_trading::MarketQueries> {
        &self.market_queries
    }
}
impl crate::features::market_data::prices::ports::PriceDecimals for FeatureContext {
    fn price_decimals(&self) -> u8 {
        self.price_decimals
    }
}
impl crate::features::market_data::prices::ports::ReferencePrices for FeatureContext {
    fn price_feed(&self) -> &Arc<dyn crate::features::market_data::prices::ports::PriceFeed> {
        &self.price_feed
    }
}
