//! HTTP application composition; business workflows are owned by features.

mod app;
mod features;
mod http;
pub use crate::features::compliance::kyc::cases::dto::KycCaseRequest;
pub use crate::features::compliance::kyc::cases::dto::KycCaseResponse;
pub use crate::features::compliance::onboarding::dto::OnboardingStatusResponse;
pub use crate::features::compliance::onboarding::dto::TermsAcceptanceRequest;
pub use crate::features::identity::access::dto::WalletAuthorizationRequest;
pub use crate::features::identity::access::dto::WalletAuthorizationResponse;
pub use crate::features::identity::accounts::dto::AuthSessionResponse;
pub use crate::features::identity::accounts::dto::PasswordLoginRequest;
pub use crate::features::identity::accounts::dto::PasswordRegisterRequest;
pub use crate::features::identity::accounts::dto::SignupAccountType;
pub use crate::features::identity::sessions::dto::CurrentUserResponse;
pub use crate::features::identity::wallets::linking::dto::SolanaWalletChallengeRequest;
pub use crate::features::identity::wallets::linking::dto::WalletChallengeRequest;
pub use crate::features::identity::wallets::linking::dto::WalletChallengeResponse;
pub use crate::features::identity::wallets::linking::dto::WalletLinkRequest;
pub use crate::features::identity::wallets::linking::dto::WalletLinkResponse;
pub use crate::features::issuer::assets::backing::dto::SubmitBackingRequest;
pub use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
pub use crate::features::issuer::assets::drafts::dto::CreateAssetDraftRequest;
pub use crate::features::issuer::assets::issuance::inventory::dto::InitialInventoryResponse;
pub use crate::features::issuer::assets::issuance::setup::dto::AssetSetupOperationResponse;
pub use crate::features::issuer::assets::issuance::setup::dto::SetupStageResponse;
pub use crate::features::issuer::assets::media::dto::ConfirmImageUploadRequest;
pub use crate::features::issuer::assets::media::dto::ImageUploadRequest;
pub use crate::features::issuer::assets::media::dto::ImageUploadTicket;
pub use crate::features::issuer::organizations::dto::IssuerOnboardingRequest;
pub use crate::features::issuer::organizations::dto::IssuerOrganizationResponse;
pub use crate::features::issuer::organizations::dto::IssuerOverviewResponse;
pub use crate::features::settlement::direct_purchases::catalog::dto::InvestorCatalogAssetResponse;
pub use crate::features::settlement::direct_purchases::catalog::dto::InvestorCatalogResponse;
pub use crate::http::models::ApiErrorResponse;
pub use crate::http::models::HealthResponse;
pub use app::bootstrap::router_from_env;
pub use app::config::ServerConfig;
pub(crate) use app::constants::SESSION_RATE_LIMIT;
pub(crate) use app::router::router_with_state;
pub use app::router::{fail_closed_router, router};
pub use app::state::GatewayState;
pub use app::state::GatewayStateBuilder;
pub use features::catalog::instruments::{
    Instrument, InstrumentCatalog, InstrumentOrigin, InstrumentProvider,
    SimulatedInstrumentProvider, UnifiedInstrumentCatalog,
};
pub(crate) use features::identity::access::actor::AuthenticatedUser;
pub(crate) use http::errors::ApiError;

pub use crate::features::market_data::assets::dto::AssetResponse;
pub use crate::features::market_data::charts::dto::NavPointResponse;
pub use crate::features::market_data::charts::dto::PricePointResponse;
pub use crate::features::market_data::markets::dto::MarketDetailResponse;
pub use crate::features::market_data::markets::dto::MarketListResponse;
pub use crate::features::market_data::markets::dto::MarketResponse;
pub use crate::features::market_data::markets::dto::OrderBookLevelResponse;
pub use crate::features::market_data::markets::dto::OrderBookResponse;
pub use crate::features::market_data::markets::dto::PublicTradeResponse;
pub use crate::features::portfolio::activity::dto::ActivityEventResponse;
pub use crate::features::portfolio::performance::dto::PerformancePointResponse;
pub use crate::features::portfolio::valuation::dto::HoldingResponse;
pub use crate::features::portfolio::valuation::dto::PortfolioSummaryResponse;
pub use crate::features::settlement::direct_purchases::preparation::dto::PreparePurchaseRequest;
pub use crate::features::settlement::direct_purchases::preparation::dto::PreparedPurchaseResponse;
pub use crate::features::settlement::direct_purchases::quotes::dto::QuoteRequest;
pub use crate::features::settlement::direct_purchases::quotes::dto::QuoteResponse;
pub use crate::features::settlement::direct_purchases::quotes::dto::QuoteSide;
pub use crate::features::settlement::direct_purchases::submission::dto::PurchaseSettlementResponse;
pub use crate::features::settlement::direct_purchases::submission::dto::SubmitPurchaseRequest;
pub use crate::http::models::Data;

pub use app::chain::ChainProviders;
pub use app::persistence::Persistence;
