use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    asset_queries: Arc<dyn issuer_assets::assets::drafts::ports::AssetQueries>,
    auth_service: Arc<identity::BrokerageAuthService>,
    inventory: Arc<dyn issuer_assets::assets::issuance::inventory::ports::InventoryProvider>,
    payment_assets: Arc<dyn crate::features::payments::cngn::ports::PaymentAssetRegistry>,
    positions: Arc<dyn issuer_assets::assets::positions::ports::PositionProvider>,
    price_feed: Arc<dyn crate::features::market_data::prices::ports::PriceFeed>,
    purchase_preparation:
        Arc<dyn settlement_orchestrator::direct_purchases::preparation::ports::PurchasePreparation>,
    quote_config: settlement_orchestrator::direct_purchases::quotes::models::QuoteConfig,
    rate_limiter: Arc<dyn identity::access::rate_limiter::RateLimiter>,
    setup_queries: Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupQueries>,
    wallet_queries: Arc<dyn identity::wallets::ports::WalletQueries>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            asset_queries: state.asset_queries.clone(),
            auth_service: state.auth_service.clone(),
            inventory: state.inventory.clone(),
            payment_assets: state.payment_assets.clone(),
            positions: state.positions.clone(),
            price_feed: state.price_feed.clone(),
            purchase_preparation: state.purchase_preparation.clone(),
            quote_config: state.quote_config,
            rate_limiter: state.rate_limiter.clone(),
            setup_queries: state.setup_queries.clone(),
            wallet_queries: state.wallet_queries.clone(),
        }
    }
}
impl crate::features::identity::accounts::ports::AccountLookup for FeatureContext {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries> {
        &self.account_queries
    }
}
impl crate::features::issuer::assets::listings::ports::PublishedAssets for FeatureContext {
    fn asset_queries(&self) -> &Arc<dyn issuer_assets::assets::drafts::ports::AssetQueries> {
        &self.asset_queries
    }
}
impl crate::features::identity::access::ports::AccessPolicy for FeatureContext {
    fn auth_service(&self) -> &Arc<identity::BrokerageAuthService> {
        &self.auth_service
    }
}
impl crate::features::issuer::assets::issuance::inventory::ports::Inventory for FeatureContext {
    fn inventory(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::inventory::ports::InventoryProvider> {
        &self.inventory
    }
}
impl crate::features::payments::cngn::ports::PaymentProvider for FeatureContext {
    fn payment_assets(
        &self,
    ) -> &Arc<dyn crate::features::payments::cngn::ports::PaymentAssetRegistry> {
        &self.payment_assets
    }
}
impl crate::features::issuer::assets::issuance::inventory::ports::Positions for FeatureContext {
    fn positions(&self) -> &Arc<dyn issuer_assets::assets::positions::ports::PositionProvider> {
        &self.positions
    }
}
impl crate::features::market_data::prices::ports::ReferencePrices for FeatureContext {
    fn price_feed(&self) -> &Arc<dyn crate::features::market_data::prices::ports::PriceFeed> {
        &self.price_feed
    }
}
impl crate::features::settlement::direct_purchases::preparation::ports::Preparation
    for FeatureContext
{
    fn purchase_preparation(
        &self,
    ) -> &Arc<dyn settlement_orchestrator::direct_purchases::preparation::ports::PurchasePreparation>
    {
        &self.purchase_preparation
    }
}
impl crate::features::settlement::direct_purchases::quotes::ports::QuotePolicy for FeatureContext {
    fn quote_config(
        &self,
    ) -> settlement_orchestrator::direct_purchases::quotes::models::QuoteConfig {
        self.quote_config
    }
}
impl crate::features::identity::access::ports::RateLimit for FeatureContext {
    fn rate_limiter(&self) -> &Arc<dyn identity::access::rate_limiter::RateLimiter> {
        &self.rate_limiter
    }
}
impl crate::features::issuer::assets::issuance::setup::ports::SetupLookup for FeatureContext {
    fn setup_queries(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupQueries> {
        &self.setup_queries
    }
}
impl crate::features::identity::wallets::queries::ports::WalletLookup for FeatureContext {
    fn wallet_queries(&self) -> &Arc<dyn identity::wallets::ports::WalletQueries> {
        &self.wallet_queries
    }
}
