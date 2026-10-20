use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    assets: Arc<dyn issuer_assets::assets::drafts::ports::AssetRepository>,
    instrument_provider: Arc<dyn crate::features::catalog::instruments::ports::InstrumentProvider>,
    organizations: Arc<dyn issuer_assets::organizations::ports::OrganizationRepository>,
    price_decimals: u8,
    price_feed: Arc<dyn crate::features::market_data::prices::ports::PriceFeed>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            assets: state.assets.clone(),
            instrument_provider: state.instrument_provider.clone(),
            organizations: state.organizations.clone(),
            price_decimals: state.price_decimals,
            price_feed: state.price_feed.clone(),
        }
    }
}
impl crate::features::issuer::assets::drafts::ports::AssetStore for FeatureContext {
    fn assets(&self) -> &Arc<dyn issuer_assets::assets::drafts::ports::AssetRepository> {
        &self.assets
    }
}
impl crate::features::catalog::instruments::ports::CatalogSource for FeatureContext {
    fn instrument_provider(
        &self,
    ) -> &Arc<dyn crate::features::catalog::instruments::ports::InstrumentProvider> {
        &self.instrument_provider
    }
}
impl crate::features::issuer::organizations::ports::OrganizationStore for FeatureContext {
    fn organizations(
        &self,
    ) -> &Arc<dyn issuer_assets::organizations::ports::OrganizationRepository> {
        &self.organizations
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
