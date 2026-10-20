use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    asset_queries: Arc<dyn issuer_assets::assets::drafts::ports::AssetQueries>,
    balance_queries: Arc<dyn portfolio_trading::BalanceQueries>,
    house_half_spread_bps: u32,
    instrument_provider: Arc<dyn crate::features::catalog::instruments::ports::InstrumentProvider>,
    inventory: Arc<dyn issuer_assets::assets::issuance::inventory::ports::InventoryProvider>,
    market_queries: Arc<dyn portfolio_trading::MarketQueries>,
    order_queries: Arc<dyn portfolio_trading::OrderQueries>,
    payment_assets: Arc<dyn crate::features::payments::cngn::ports::PaymentAssetRegistry>,
    positions: Arc<dyn issuer_assets::assets::positions::ports::PositionProvider>,
    price_decimals: u8,
    price_feed: Arc<dyn crate::features::market_data::prices::ports::PriceFeed>,
    setup_queries: Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupQueries>,
    wallet_queries: Arc<dyn identity::wallets::ports::WalletQueries>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            asset_queries: state.asset_queries.clone(),
            balance_queries: state.balance_queries.clone(),
            house_half_spread_bps: state.house_half_spread_bps,
            instrument_provider: state.instrument_provider.clone(),
            inventory: state.inventory.clone(),
            market_queries: state.market_queries.clone(),
            order_queries: state.order_queries.clone(),
            payment_assets: state.payment_assets.clone(),
            positions: state.positions.clone(),
            price_decimals: state.price_decimals,
            price_feed: state.price_feed.clone(),
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
impl crate::features::portfolio::balances::ports::Balances for FeatureContext {
    fn balance_queries(&self) -> &Arc<dyn portfolio_trading::BalanceQueries> {
        &self.balance_queries
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
impl crate::features::issuer::assets::issuance::inventory::ports::Inventory for FeatureContext {
    fn inventory(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::inventory::ports::InventoryProvider> {
        &self.inventory
    }
}
impl crate::features::market_data::markets::ports::ExecutionMarket for FeatureContext {
    fn market_queries(&self) -> &Arc<dyn portfolio_trading::MarketQueries> {
        &self.market_queries
    }
}
impl crate::features::trading::orders::ports::Orders for FeatureContext {
    fn order_queries(&self) -> &Arc<dyn portfolio_trading::OrderQueries> {
        &self.order_queries
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
