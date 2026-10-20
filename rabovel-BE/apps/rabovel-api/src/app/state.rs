use crate::app::persistence::Persistence;
use crate::features::catalog::instruments::InstrumentProvider;
use crate::features::catalog::instruments::UnifiedInstrumentCatalog;
use crate::features::market_data::prices::ports::PriceFeed;
use crate::features::market_data::prices::providers::simulated::SimulatedPriceFeed;
use crate::features::market_data::prices::CatalogReferencePrices;
use crate::features::payments::cngn::ports::PaymentAssetRegistry;
use crate::features::payments::cngn::providers::disabled::DisabledPaymentAssetRegistry;
use events::EventProducer;
use identity::access::rate_limiter::InMemoryRateLimiter;
use identity::access::rate_limiter::RateLimiter;
use identity::accounts::ports::AccountRepository;
use identity::security_state::EphemeralStore;
use identity::security_state::InMemoryEphemeralStore;
use identity::wallets::linking::ports::WalletProofVerifier;
use identity::wallets::ports::WalletRepository;
use identity::wallets::providers::siwe::SiweWalletProofVerifier;
use identity::BrokerageAuthService;
use identity::BrokerageSecurityPolicy;
use infrastructure::messaging;
use issuer_assets::assets::drafts::ports::AssetRepository;
use issuer_assets::assets::issuance::setup::ports::SetupRepository;
use issuer_assets::assets::metadata::ports::MetadataStorage;
use issuer_assets::assets::metadata::providers::storage::DisabledMetadataStorage;
use issuer_assets::organizations::ports::OrganizationRepository;
use kyc::cases::ports::CaseRepository;
use kyc::KycProviderApi;
use kyc::MockKycProvider;
use portfolio_trading::EngineTradingConfig;
use portfolio_trading::EngineTradingService;
use portfolio_trading::PortfolioTradingApi;
use sha2::Digest;
use sha2::Sha256;
use std::sync::Arc;

/// Builds a [`GatewayState`] with sensible in-memory/no-op defaults for
/// anything not explicitly configured -- keeps `router()`/tests readable as
/// the state gains more collaborators (Postgres, Redis, KYC, events,
/// portfolio/trading) over time.
pub struct GatewayStateBuilder {
    pub(crate) quote_config: settlement_orchestrator::direct_purchases::quotes::models::QuoteConfig,
    pub(crate) persistence: Arc<Persistence>,
    pub(crate) sessions: Arc<dyn identity::sessions::ports::SessionRepository>,
    pub(crate) challenges: Arc<dyn identity::wallets::linking::ports::ChallengeRepository>,
    pub(crate) rate_limiter: Arc<dyn RateLimiter>,
    pub(crate) wallet_proof: Arc<dyn WalletProofVerifier>,
    pub(crate) kyc_provider: Arc<dyn KycProviderApi>,
    #[cfg(debug_assertions)]
    pub(crate) sandbox_kyc: Option<Arc<MockKycProvider>>,
    /// `None` builds the matching-engine service from `trading_config`.
    pub(crate) portfolio_trading: Option<Arc<dyn PortfolioTradingApi>>,
    pub(crate) trading_config: EngineTradingConfig,
    pub(crate) event_producer: Arc<dyn EventProducer>,
    pub(crate) metadata_storage: Arc<dyn MetadataStorage>,
    pub(crate) chain: crate::app::chain::ChainProviders,
    pub(crate) payment_assets: Arc<dyn PaymentAssetRegistry>,
    /// `None` builds the unified catalog (NGX-listed + live issuer assets)
    /// over `repository`.
    pub(crate) instrument_provider: Option<Arc<dyn InstrumentProvider>>,
    pub(crate) price_feed: Arc<dyn PriceFeed>,
    pub(crate) siwe_domain: String,
    pub(crate) public_origin: String,
    pub(crate) security_policy: Option<BrokerageSecurityPolicy>,
    pub(crate) gateway_secret: Option<[u8; 32]>,
}

impl Default for GatewayStateBuilder {
    fn default() -> Self {
        let mock_kyc = Arc::new(MockKycProvider::new(
            b"insecure-dev-only-kyc-secret".to_vec(),
        ));
        let security_state = Arc::new(InMemoryEphemeralStore::default());
        Self {
            quote_config: Default::default(),
            persistence: Arc::new(Persistence::memory()),
            sessions: security_state.clone(),
            challenges: security_state,
            rate_limiter: Arc::new(InMemoryRateLimiter::default()),
            wallet_proof: Arc::new(SiweWalletProofVerifier),
            kyc_provider: mock_kyc.clone(),
            #[cfg(debug_assertions)]
            sandbox_kyc: Some(mock_kyc),
            portfolio_trading: None,
            trading_config: EngineTradingConfig::default(),
            event_producer: Arc::new(messaging::NoopEventProducer),
            metadata_storage: Arc::new(DisabledMetadataStorage),
            chain: crate::app::chain::ChainProviders::disabled(),
            payment_assets: Arc::new(DisabledPaymentAssetRegistry),
            instrument_provider: None,
            price_feed: Arc::new(SimulatedPriceFeed),
            siwe_domain: "localhost:3000".to_string(),
            public_origin: "http://localhost:3000".to_string(),
            security_policy: None,
            gateway_secret: None,
        }
    }
}

impl GatewayStateBuilder {
    pub fn persistence(mut self, persistence: Arc<Persistence>) -> Self {
        self.persistence = persistence;
        self
    }
    pub fn ephemeral(mut self, ephemeral: Arc<dyn EphemeralStore>) -> Self {
        self.sessions = ephemeral.clone();
        self.challenges = ephemeral;
        self
    }
    pub fn rate_limiter(mut self, rate_limiter: Arc<dyn RateLimiter>) -> Self {
        self.rate_limiter = rate_limiter;
        self
    }

    pub(crate) fn wallet_proof(mut self, wallet_proof: Arc<dyn WalletProofVerifier>) -> Self {
        self.wallet_proof = wallet_proof;
        self
    }
    /// Sets both the trait-object KYC provider and the sandbox-only
    /// downcast used by `POST /kyc/_sandbox/advance`, so the debug-only
    /// route can always drive whatever mock instance is actually configured.
    pub fn mock_kyc_provider(mut self, provider: Arc<MockKycProvider>) -> Self {
        #[cfg(debug_assertions)]
        {
            self.sandbox_kyc = Some(provider.clone());
        }
        self.kyc_provider = provider;
        self
    }
    pub fn portfolio_trading(mut self, portfolio_trading: Arc<dyn PortfolioTradingApi>) -> Self {
        self.portfolio_trading = Some(portfolio_trading);
        self
    }
    /// House limits and float for the default matching-engine service.
    pub fn trading_config(mut self, config: EngineTradingConfig) -> Self {
        self.trading_config = config;
        self
    }
    pub fn event_producer(mut self, event_producer: Arc<dyn EventProducer>) -> Self {
        self.event_producer = event_producer;
        self
    }
    pub fn metadata_storage(mut self, metadata_storage: Arc<dyn MetadataStorage>) -> Self {
        self.metadata_storage = metadata_storage;
        self
    }
    pub fn mint_setup<
        T: issuer_assets::assets::issuance::setup::ports::MintSetupExecutor
            + issuer_assets::assets::issuance::inventory::ports::InventoryProvider
            + issuer_assets::assets::positions::ports::PositionProvider
            + settlement_orchestrator::direct_purchases::preparation::ports::PurchasePreparation
            + settlement_orchestrator::direct_purchases::submission::ports::PurchaseSubmission
            + 'static,
    >(
        mut self,
        provider: Arc<T>,
    ) -> Self {
        self.chain = crate::app::chain::ChainProviders::from_shared(provider);
        self
    }
    pub fn chain(mut self, chain: crate::app::chain::ChainProviders) -> Self {
        self.chain = chain;
        self
    }
    pub fn payment_assets(mut self, payment_assets: Arc<dyn PaymentAssetRegistry>) -> Self {
        self.payment_assets = payment_assets;
        self
    }
    pub fn instrument_provider(mut self, provider: Arc<dyn InstrumentProvider>) -> Self {
        self.instrument_provider = Some(provider);
        self
    }
    pub fn siwe_domain(mut self, siwe_domain: impl Into<String>) -> Self {
        self.siwe_domain = siwe_domain.into();
        self
    }
    pub fn public_origin(mut self, public_origin: impl Into<String>) -> Self {
        self.public_origin = public_origin.into();
        self
    }

    pub fn gateway_secret(mut self, secret: &str) -> Self {
        self.gateway_secret = Some(Sha256::digest(secret.as_bytes()).into());
        self
    }

    pub fn security_policy(mut self, policy: BrokerageSecurityPolicy) -> Self {
        self.security_policy = Some(policy);
        self
    }

    pub fn quote_config(
        mut self,
        config: settlement_orchestrator::direct_purchases::quotes::models::QuoteConfig,
    ) -> Self {
        self.quote_config = config;
        self
    }
    pub fn build(self) -> GatewayState {
        let auth_service = if let Some(policy) = self.security_policy {
            Arc::new(BrokerageAuthService::new(policy))
        } else {
            Arc::new(BrokerageAuthService::new(BrokerageSecurityPolicy::default()))
        };
        let instrument_provider = self.instrument_provider.unwrap_or_else(|| {
            Arc::new(UnifiedInstrumentCatalog::new(
                self.persistence.assets.clone(),
            ))
        });
        let house_half_spread_bps = self.trading_config.engine.default_limits.half_spread_bps;
        let price_decimals = self.trading_config.price_decimals;
        let portfolio_trading = self.portfolio_trading.unwrap_or_else(|| {
            let prices =
                CatalogReferencePrices::new(instrument_provider.clone(), self.price_feed.clone());
            Arc::new(EngineTradingService::new(
                Arc::new(prices),
                self.trading_config,
            ))
        });
        let order_use_cases = crate::features::trading::compose(
            portfolio_trading.clone(),
            self.persistence.accounts.clone(),
            self.persistence.wallets.clone(),
            auth_service.clone(),
            self.rate_limiter.clone(),
            self.event_producer.clone(),
        );
        GatewayState {
            account_queries: self.persistence.accounts.clone(),
            wallet_queries: self.persistence.wallets.clone(),
            profile_updates: self.persistence.accounts.clone(),
            asset_queries: self.persistence.assets.clone(),
            setup_queries: self.persistence.setups.clone(),
            quote_config: self.quote_config,
            auth_service,
            accounts: self.persistence.accounts.clone(),
            wallets: self.persistence.wallets.clone(),
            organizations: self.persistence.organizations.clone(),
            assets: self.persistence.assets.clone(),
            setups: self.persistence.setups.clone(),
            cases: self.persistence.cases.clone(),
            sessions: self.sessions,
            challenges: self.challenges,
            rate_limiter: self.rate_limiter,
            wallet_proof: self.wallet_proof,
            kyc_provider: self.kyc_provider,
            #[cfg(debug_assertions)]
            sandbox_kyc: self.sandbox_kyc,
            balance_queries: portfolio_trading.clone(),
            order_queries: portfolio_trading.clone(),
            market_queries: portfolio_trading.clone(),
            order_use_cases,
            event_producer: self.event_producer,
            metadata_storage: self.metadata_storage,
            mint_setup: self.chain.mint_setup,
            inventory: self.chain.inventory,
            positions: self.chain.positions,
            purchase_preparation: self.chain.purchase_preparation,
            purchase_submission: self.chain.purchase_submission,
            payment_assets: self.payment_assets,
            instrument_provider,
            price_feed: self.price_feed,
            house_half_spread_bps,
            price_decimals,
            siwe_domain: self.siwe_domain.into(),
            public_origin: self.public_origin.into(),
            gateway_secret: self.gateway_secret,
        }
    }
}

#[derive(Clone)]
pub struct GatewayState {
    pub(crate) account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    pub(crate) wallet_queries: Arc<dyn identity::wallets::ports::WalletQueries>,
    pub(crate) profile_updates: Arc<dyn identity::accounts::ports::ProfileUpdates>,
    pub(crate) asset_queries: Arc<dyn issuer_assets::assets::drafts::ports::AssetQueries>,
    pub(crate) setup_queries: Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupQueries>,

    pub(crate) quote_config: settlement_orchestrator::direct_purchases::quotes::models::QuoteConfig,
    pub(crate) auth_service: Arc<BrokerageAuthService>,
    pub(crate) accounts: Arc<dyn AccountRepository>,
    pub(crate) wallets: Arc<dyn WalletRepository>,
    pub(crate) organizations: Arc<dyn OrganizationRepository>,
    pub(crate) assets: Arc<dyn AssetRepository>,
    pub(crate) setups: Arc<dyn SetupRepository>,
    pub(crate) cases: Arc<dyn CaseRepository>,
    pub(crate) sessions: Arc<dyn identity::sessions::ports::SessionRepository>,
    pub(crate) challenges: Arc<dyn identity::wallets::linking::ports::ChallengeRepository>,
    pub(crate) rate_limiter: Arc<dyn RateLimiter>,
    pub(crate) wallet_proof: Arc<dyn WalletProofVerifier>,
    pub(crate) kyc_provider: Arc<dyn KycProviderApi>,
    #[cfg(debug_assertions)]
    pub(crate) sandbox_kyc: Option<Arc<MockKycProvider>>,
    pub(crate) balance_queries: Arc<dyn portfolio_trading::BalanceQueries>,
    pub(crate) order_queries: Arc<dyn portfolio_trading::OrderQueries>,
    pub(crate) market_queries: Arc<dyn portfolio_trading::MarketQueries>,
    pub(crate) order_use_cases: Arc<crate::features::trading::TradingUseCases>,
    pub(crate) event_producer: Arc<dyn EventProducer>,
    pub(crate) metadata_storage: Arc<dyn MetadataStorage>,
    pub(crate) mint_setup:
        Arc<dyn issuer_assets::assets::issuance::setup::ports::MintSetupExecutor>,
    pub(crate) inventory:
        Arc<dyn issuer_assets::assets::issuance::inventory::ports::InventoryProvider>,
    pub(crate) positions: Arc<dyn issuer_assets::assets::positions::ports::PositionProvider>,
    pub(crate) purchase_preparation:
        Arc<dyn settlement_orchestrator::direct_purchases::preparation::ports::PurchasePreparation>,
    pub(crate) purchase_submission:
        Arc<dyn settlement_orchestrator::direct_purchases::submission::ports::PurchaseSubmission>,
    pub(crate) payment_assets: Arc<dyn PaymentAssetRegistry>,
    pub(crate) instrument_provider: Arc<dyn InstrumentProvider>,
    pub(crate) price_feed: Arc<dyn PriceFeed>,
    /// House quote half-spread, shown as bid/ask on `/markets`.
    pub(crate) house_half_spread_bps: u32,
    /// Payment-asset decimals prices are quoted in.
    pub(crate) price_decimals: u8,
    pub(crate) siwe_domain: Arc<str>,
    pub(crate) public_origin: Arc<str>,
    pub(crate) gateway_secret: Option<[u8; 32]>,
}

impl GatewayState {
    pub(crate) fn disabled() -> Self {
        GatewayStateBuilder::default().build()
    }
}
