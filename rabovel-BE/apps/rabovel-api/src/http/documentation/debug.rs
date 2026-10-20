use crate::app::state::GatewayState;
use crate::features::catalog::instruments::Instrument;
use crate::features::catalog::instruments::InstrumentCatalog;
use crate::features::catalog::instruments::InstrumentOrigin;
use crate::features::compliance::kyc::sandbox::handlers::kyc_sandbox_advance_handler;
use crate::features::compliance::onboarding::dto::OnboardingStatusResponse;
use crate::features::compliance::onboarding::dto::TermsAcceptanceRequest;
use crate::features::identity::access::dto::WalletAuthorizationRequest;
use crate::features::identity::access::dto::WalletAuthorizationResponse;
use crate::features::identity::accounts::dto::AuthSessionResponse;
use crate::features::identity::accounts::dto::PasswordLoginRequest;
use crate::features::identity::accounts::dto::PasswordRegisterRequest;
use crate::features::identity::accounts::dto::SignupAccountType;
use crate::features::identity::sessions::dto::CurrentUserResponse;
use crate::features::identity::wallets::linking::dto::SolanaWalletChallengeRequest;
use crate::features::identity::wallets::linking::dto::WalletChallengeRequest;
use crate::features::identity::wallets::linking::dto::WalletChallengeResponse;
use crate::features::identity::wallets::linking::dto::WalletLinkRequest;
use crate::features::identity::wallets::linking::dto::WalletLinkResponse;
use crate::features::issuer::assets::backing::dto::SubmitBackingRequest;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::features::issuer::assets::drafts::dto::CreateAssetDraftRequest;
use crate::features::issuer::assets::issuance::inventory::dto::InitialInventoryResponse;
use crate::features::issuer::assets::issuance::setup::dto::AssetSetupOperationResponse;
use crate::features::issuer::assets::issuance::setup::dto::SetupStageResponse;
use crate::features::issuer::organizations::dto::IssuerOnboardingRequest;
use crate::features::issuer::organizations::dto::IssuerOrganizationResponse;
use crate::features::issuer::organizations::dto::IssuerOverviewResponse;
use crate::features::market_data::markets::dto::MarketDetailResponse;
use crate::features::market_data::markets::dto::MarketListResponse;
use crate::features::market_data::markets::dto::MarketResponse;
use crate::features::payments::cngn::models::PaymentAssetStatus;
use crate::features::payments::cngn::models::WalletPaymentAssetStatus;
use crate::features::settlement::direct_purchases::catalog::dto::InvestorCatalogAssetResponse;
use crate::features::settlement::direct_purchases::catalog::dto::InvestorCatalogResponse;
use crate::features::settlement::direct_purchases::preparation::dto::PreparePurchaseRequest;
use crate::features::settlement::direct_purchases::preparation::dto::PreparedPurchaseResponse;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteRequest;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteResponse;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteSide;
use crate::features::settlement::direct_purchases::submission::dto::PurchaseSettlementResponse;
use crate::features::settlement::direct_purchases::submission::dto::SubmitPurchaseRequest;
use crate::http::models::ApiErrorResponse;
use crate::http::models::HealthResponse;
use axum::response::Html;
use axum::response::Json;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use identity::BrokerageOnboardingStatus;
use identity::IdentityProfile;
use identity::WalletConnection;
use issuer_assets::assets::drafts::models::BackingEvidence;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::app::health::health_handler,
        crate::features::identity::accounts::handlers::password_register_handler,
        crate::features::identity::accounts::handlers::password_login_handler,
        crate::features::identity::sessions::handlers::auth_me_handler,
        crate::features::identity::sessions::handlers::logout_handler,
        crate::features::issuer::organizations::handlers::issuer_overview_handler,
        crate::features::issuer::organizations::handlers::issuer_onboarding_handler,
        crate::features::issuer::assets::drafts::handlers::create_asset_draft_handler,
        crate::features::issuer::assets::drafts::handlers::list_asset_drafts_handler,
        crate::features::issuer::assets::drafts::handlers::get_asset_draft_handler,
        crate::features::issuer::assets::drafts::handlers::update_asset_draft_handler,
        crate::features::issuer::assets::drafts::handlers::delete_asset_draft_handler,
        crate::features::issuer::assets::review::handlers::submit_asset_for_demo_review_handler,
        crate::features::issuer::assets::issuance::setup::handlers::start_asset_setup_handler,
        crate::features::issuer::assets::issuance::setup::handlers::get_asset_setup_operation_handler,
        crate::features::issuer::assets::issuance::inventory::handlers::get_initial_inventory_handler,
        crate::features::issuer::assets::issuance::inventory::handlers::issue_initial_inventory_handler,
        crate::features::issuer::assets::backing::handlers::submit_asset_backing_handler,
        crate::features::issuer::assets::listings::handlers::publish_asset_listing_handler,
        crate::features::payments::cngn::broker_accounts::handlers::cngn_payment_asset_handler,
        crate::features::payments::cngn::broker_accounts::handlers::create_broker_cngn_account_handler,
        crate::features::compliance::onboarding::handlers::onboarding_status_handler,
        crate::features::compliance::onboarding::handlers::accept_terms_handler,
        crate::features::identity::wallets::linking::handlers::wallet_challenge_handler,
        crate::features::identity::wallets::linking::handlers::solana_wallet_challenge_handler,
        crate::features::identity::wallets::linking::handlers::wallet_link_handler,
        crate::features::identity::wallets::queries::handlers::list_wallets_handler,
        crate::features::identity::access::handlers::authorize_wallet_handler,
        crate::features::payments::cngn::wallet_accounts::handlers::investor_cngn_account_handler,
        crate::features::payments::cngn::wallet_accounts::handlers::create_investor_cngn_account_handler,
        crate::features::settlement::direct_purchases::catalog::handlers::investor_catalog_handler,
        crate::features::catalog::instruments::handlers::instruments_handler,
        crate::features::market_data::markets::handlers::list_markets_handler,
        crate::features::market_data::markets::handlers::get_market_handler,
        crate::features::settlement::direct_purchases::preparation::handlers::prepare_investor_purchase_handler,
        crate::features::settlement::direct_purchases::submission::handlers::submit_investor_purchase_handler
    ),
    components(schemas(
        HealthResponse,
        ApiErrorResponse,
        PasswordRegisterRequest,
        SignupAccountType,
        PasswordLoginRequest,
        AuthSessionResponse,
        CurrentUserResponse,
        IssuerOverviewResponse,
        IssuerOrganizationResponse,
        IssuerOnboardingRequest,
        CreateAssetDraftRequest,
        AssetDraftResponse,
        AssetSetupOperationResponse,
        InitialInventoryResponse,
        SubmitBackingRequest,
        BackingEvidence,
        SetupStageResponse,
        PaymentAssetStatus,
        WalletPaymentAssetStatus,
        InvestorCatalogAssetResponse,
        InvestorCatalogResponse,
        Instrument,
        InstrumentCatalog,
        InstrumentOrigin,
        MarketResponse,
        MarketListResponse,
        MarketDetailResponse,
        QuoteRequest,
        QuoteResponse,
        QuoteSide,
        PreparePurchaseRequest,
        PreparedPurchaseResponse,
        SubmitPurchaseRequest,
        PurchaseSettlementResponse,
        OnboardingStatusResponse,
        TermsAcceptanceRequest,
        WalletChallengeRequest,
        SolanaWalletChallengeRequest,
        WalletChallengeResponse,
        WalletLinkRequest,
        WalletLinkResponse,
        WalletAuthorizationRequest,
        WalletAuthorizationResponse,
        IdentityProfile,
        WalletConnection,
        BrokerageOnboardingStatus
    ))
)]
pub(crate) struct ApiDoc;

pub(crate) fn add_debug_routes(router: Router<GatewayState>) -> Router<GatewayState> {
    router
        .route("/kyc/_sandbox/advance", post(kyc_sandbox_advance_handler))
        .route("/swagger-ui", get(swagger_ui_handler))
        .route("/api-docs/openapi.json", get(openapi_handler))
}

pub(crate) async fn openapi_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub(crate) async fn swagger_ui_handler() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>Rabovel API</title>
<link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head>
<body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>SwaggerUIBundle({url:'/api-docs/openapi.json',dom_id:'#swagger-ui'});</script></body></html>"#,
    )
}
