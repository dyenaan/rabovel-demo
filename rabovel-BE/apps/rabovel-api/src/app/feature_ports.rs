use std::sync::Arc;
impl crate::features::identity::accounts::ports::AccountStore for crate::app::state::GatewayState {
    fn accounts(&self) -> &Arc<dyn identity::accounts::ports::AccountRepository> {
        &self.accounts
    }
}
impl crate::features::identity::wallets::queries::ports::WalletStore
    for crate::app::state::GatewayState
{
    fn wallets(&self) -> &Arc<dyn identity::wallets::ports::WalletRepository> {
        &self.wallets
    }
}
impl crate::features::issuer::organizations::ports::OrganizationStore
    for crate::app::state::GatewayState
{
    fn organizations(
        &self,
    ) -> &Arc<dyn issuer_assets::organizations::ports::OrganizationRepository> {
        &self.organizations
    }
}
impl crate::features::issuer::assets::drafts::ports::AssetStore
    for crate::app::state::GatewayState
{
    fn assets(&self) -> &Arc<dyn issuer_assets::assets::drafts::ports::AssetRepository> {
        &self.assets
    }
}
impl crate::features::issuer::assets::issuance::setup::ports::SetupStore
    for crate::app::state::GatewayState
{
    fn setups(&self) -> &Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupRepository> {
        &self.setups
    }
}
impl crate::features::compliance::kyc::cases::ports::CaseStore for crate::app::state::GatewayState {
    fn cases(&self) -> &Arc<dyn kyc::cases::ports::CaseRepository> {
        &self.cases
    }
}
impl crate::features::identity::access::ports::RateLimit for crate::app::state::GatewayState {
    fn rate_limiter(&self) -> &Arc<dyn identity::access::rate_limiter::RateLimiter> {
        &self.rate_limiter
    }
}
impl crate::features::identity::access::ports::AccessPolicy for crate::app::state::GatewayState {
    fn auth_service(&self) -> &Arc<identity::BrokerageAuthService> {
        &self.auth_service
    }
}
impl crate::features::identity::wallets::linking::ports::WalletProof
    for crate::app::state::GatewayState
{
    fn wallet_proof(&self) -> &Arc<dyn identity::wallets::linking::ports::WalletProofVerifier> {
        &self.wallet_proof
    }
}
impl crate::features::compliance::kyc::cases::ports::KycProvider
    for crate::app::state::GatewayState
{
    fn kyc_provider(&self) -> &Arc<dyn kyc::KycProviderApi> {
        &self.kyc_provider
    }
}
#[cfg(debug_assertions)]
impl crate::features::compliance::kyc::sandbox::ports::SandboxKyc
    for crate::app::state::GatewayState
{
    fn sandbox_kyc(&self) -> &Option<Arc<kyc::MockKycProvider>> {
        &self.sandbox_kyc
    }
}
impl crate::features::compliance::onboarding::ports::EventPublisher
    for crate::app::state::GatewayState
{
    fn event_producer(&self) -> &Arc<dyn events::EventProducer> {
        &self.event_producer
    }
}
impl crate::features::issuer::assets::media::ports::MetadataProvider
    for crate::app::state::GatewayState
{
    fn metadata_storage(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::metadata::ports::MetadataStorage> {
        &self.metadata_storage
    }
}
impl crate::features::issuer::assets::issuance::setup::ports::IssuanceProvider
    for crate::app::state::GatewayState
{
    fn mint_setup(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::setup::ports::MintSetupExecutor> {
        &self.mint_setup
    }
}
impl crate::features::payments::cngn::ports::PaymentProvider for crate::app::state::GatewayState {
    fn payment_assets(
        &self,
    ) -> &Arc<dyn crate::features::payments::cngn::ports::PaymentAssetRegistry> {
        &self.payment_assets
    }
}
impl crate::features::catalog::instruments::ports::CatalogSource
    for crate::app::state::GatewayState
{
    fn instrument_provider(
        &self,
    ) -> &Arc<dyn crate::features::catalog::instruments::ports::InstrumentProvider> {
        &self.instrument_provider
    }
}
impl crate::features::market_data::prices::ports::ReferencePrices
    for crate::app::state::GatewayState
{
    fn price_feed(&self) -> &Arc<dyn crate::features::market_data::prices::ports::PriceFeed> {
        &self.price_feed
    }
}
impl crate::features::trading::ports::HouseSpread for crate::app::state::GatewayState {
    fn house_half_spread_bps(&self) -> u32 {
        self.house_half_spread_bps
    }
}
impl crate::features::market_data::prices::ports::PriceDecimals
    for crate::app::state::GatewayState
{
    fn price_decimals(&self) -> u8 {
        self.price_decimals
    }
}
impl crate::features::identity::wallets::linking::ports::WalletDomain
    for crate::app::state::GatewayState
{
    fn siwe_domain(&self) -> &Arc<str> {
        &self.siwe_domain
    }
}
impl crate::features::identity::wallets::linking::ports::WalletOrigin
    for crate::app::state::GatewayState
{
    fn public_origin(&self) -> &Arc<str> {
        &self.public_origin
    }
}
impl crate::features::issuer::assets::issuance::inventory::ports::Inventory
    for crate::app::state::GatewayState
{
    fn inventory(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::inventory::ports::InventoryProvider> {
        &self.inventory
    }
}
impl crate::features::issuer::assets::issuance::inventory::ports::Positions
    for crate::app::state::GatewayState
{
    fn positions(&self) -> &Arc<dyn issuer_assets::assets::positions::ports::PositionProvider> {
        &self.positions
    }
}
impl crate::features::settlement::direct_purchases::preparation::ports::Preparation
    for crate::app::state::GatewayState
{
    fn purchase_preparation(
        &self,
    ) -> &Arc<dyn settlement_orchestrator::direct_purchases::preparation::ports::PurchasePreparation>
    {
        &self.purchase_preparation
    }
}
impl crate::features::settlement::direct_purchases::submission::ports::Submission
    for crate::app::state::GatewayState
{
    fn purchase_submission(
        &self,
    ) -> &Arc<dyn settlement_orchestrator::direct_purchases::submission::ports::PurchaseSubmission>
    {
        &self.purchase_submission
    }
}
impl crate::features::identity::sessions::ports::SessionStore for crate::app::state::GatewayState {
    fn sessions(&self) -> &Arc<dyn identity::sessions::ports::SessionRepository> {
        &self.sessions
    }
}
impl crate::features::identity::wallets::linking::ports::ChallengeStore
    for crate::app::state::GatewayState
{
    fn challenges(&self) -> &Arc<dyn identity::wallets::linking::ports::ChallengeRepository> {
        &self.challenges
    }
}
impl crate::features::portfolio::balances::ports::Balances for crate::app::state::GatewayState {
    fn balance_queries(&self) -> &Arc<dyn portfolio_trading::BalanceQueries> {
        &self.balance_queries
    }
}
impl crate::features::trading::orders::ports::Orders for crate::app::state::GatewayState {
    fn order_queries(&self) -> &Arc<dyn portfolio_trading::OrderQueries> {
        &self.order_queries
    }
}
impl crate::features::market_data::markets::ports::ExecutionMarket
    for crate::app::state::GatewayState
{
    fn market_queries(&self) -> &Arc<dyn portfolio_trading::MarketQueries> {
        &self.market_queries
    }
}
impl crate::features::identity::accounts::ports::AccountLookup for crate::app::state::GatewayState {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries> {
        &self.account_queries
    }
}
impl crate::features::identity::wallets::queries::ports::WalletLookup
    for crate::app::state::GatewayState
{
    fn wallet_queries(&self) -> &Arc<dyn identity::wallets::ports::WalletQueries> {
        &self.wallet_queries
    }
}
impl crate::features::identity::accounts::ports::ProfileUpdates
    for crate::app::state::GatewayState
{
    fn profile_updates(&self) -> &Arc<dyn identity::accounts::ports::ProfileUpdates> {
        &self.profile_updates
    }
}
impl crate::features::issuer::assets::listings::ports::PublishedAssets
    for crate::app::state::GatewayState
{
    fn asset_queries(&self) -> &Arc<dyn issuer_assets::assets::drafts::ports::AssetQueries> {
        &self.asset_queries
    }
}
impl crate::features::issuer::assets::issuance::setup::ports::SetupLookup
    for crate::app::state::GatewayState
{
    fn setup_queries(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupQueries> {
        &self.setup_queries
    }
}
impl crate::features::settlement::direct_purchases::quotes::ports::QuotePolicy
    for crate::app::state::GatewayState
{
    fn quote_config(
        &self,
    ) -> settlement_orchestrator::direct_purchases::quotes::models::QuoteConfig {
        self.quote_config
    }
}
