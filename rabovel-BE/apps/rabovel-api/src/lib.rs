mod bootstrap;
mod config;
mod http;
mod mint_setup;
mod payment_assets;
mod price_feed;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Path, Request, State},
    http::{request::Parts, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json},
    Router,
};
#[cfg(debug_assertions)]
use axum::{
    response::Html,
    routing::{get, post},
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
pub use bootstrap::router_from_env;
pub use config::ServerConfig;
use domain::assets::{BackingEvidence, NewAssetDraft};
#[cfg(debug_assertions)]
use domain::auth::IdentityProfile;
use domain::auth::{
    AuthError, AuthPrincipal, BrokerageAuthService, BrokerageOnboardingStatus,
    BrokerageSecurityPolicy, RiskTier, SupportedChain, UserRole, VerifiedGoogleIdentity,
    WalletAction, WalletConnection, WalletProvider,
};
use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
use events::schema::{
    kyc::{KycCaseSubmitted, KycRiskTier, KycVerdictReceived},
    onboarding::{OnboardingStatus as EventOnboardingStatus, OnboardingStatusChanged},
    tokenization::TokenizationRequested,
    trading::{OrderAccepted, OrderSide as EventOrderSide},
    wallet::WalletLinked,
};
use infrastructure::messaging::{self, EventProducer};
use infrastructure::metadata_storage::{DisabledMetadataStorage, MetadataStorage};
use kyc::{KycError, KycProviderApi, KycSubmission, MockKycProvider};
use portfolio_trading::{
    NewOrderRequest, OrderAck, OrderSide, OrderStatus, PortfolioSnapshot, PortfolioTradingApi,
    StubPortfolioTradingService,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use siwe::Message;
use std::{
    collections::HashSet,
    env,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};
#[cfg(debug_assertions)]
use utoipa::OpenApi;
use uuid::Uuid;

#[cfg(test)]
use identity::provider::OidcStart;
use identity::provider::{GoogleIdentityProvider, OidcPending, WalletProofVerifier};
use identity::{
    AssetDraftRecord, AssetSetupOperation, InMemoryRepository, IssuerOrganization, KycCaseRecord,
    KycCaseStatus, PasswordCredential, Repository, SetupStageRecord, UserRecord,
};
use infrastructure::ephemeral::{
    EphemeralStore, InMemoryEphemeralStore, OidcPendingRecord, SessionRecord, WalletChallengeRecord,
};
#[cfg(test)]
use infrastructure::metadata_storage::InMemoryMetadataStorage;
use infrastructure::providers::{DisabledGoogleIdentityProvider, SiweWalletProofVerifier};
use infrastructure::rate_limiter::{InMemoryRateLimiter, RateLimiter};
use mint_setup::{DisabledMintSetupExecutor, InitialInventory, MintSetupExecutor, PurchaseTerms};
#[cfg(test)]
use mint_setup::{InvestorAssetPosition, SetupExecution, SetupExecutionError};
use payment_assets::{
    DisabledPaymentAssetRegistry, PaymentAssetRegistry, PaymentAssetStatus,
    WalletPaymentAssetStatus,
};
use price_feed::{PriceFeed, SimulatedPriceFeed};

pub use http::dto::*;

const OIDC_CHALLENGE_TTL_SECONDS: u64 = 300;
const WALLET_CHALLENGE_TTL_SECONDS: u64 = 300;
const SESSION_TTL_SECONDS: u64 = 900;
/// Applied per authenticated session on every financial-action route.
const SESSION_RATE_LIMIT: (u32, u64) = (10, 60);
/// Applied per client IP (as reported by the WAF in front of this service --
/// see the `waf` docker-compose service) across all routes.
const IP_RATE_LIMIT: (u32, u64) = (60, 60);

/// Builds a [`GatewayState`] with sensible in-memory/no-op defaults for
/// anything not explicitly configured -- keeps `router()`/tests readable as
/// the state gains more collaborators (Postgres, Redis, KYC, events,
/// portfolio/trading) over time.
pub struct GatewayStateBuilder {
    repository: Arc<dyn Repository>,
    ephemeral: Arc<dyn EphemeralStore>,
    rate_limiter: Arc<dyn RateLimiter>,
    google: Arc<dyn GoogleIdentityProvider>,
    wallet_proof: Arc<dyn WalletProofVerifier>,
    kyc_provider: Arc<dyn KycProviderApi>,
    #[cfg(debug_assertions)]
    sandbox_kyc: Option<Arc<MockKycProvider>>,
    portfolio_trading: Arc<dyn PortfolioTradingApi>,
    event_producer: Arc<dyn EventProducer>,
    metadata_storage: Arc<dyn MetadataStorage>,
    mint_setup: Arc<dyn MintSetupExecutor>,
    payment_assets: Arc<dyn PaymentAssetRegistry>,
    price_feed: Arc<dyn PriceFeed>,
    siwe_domain: String,
    public_origin: String,
    trusted_role_assignments: Vec<TrustedRoleAssignment>,
    trusted_email_roles: Vec<(String, UserRole)>,
    security_policy: Option<BrokerageSecurityPolicy>,
}

impl Default for GatewayStateBuilder {
    fn default() -> Self {
        let mock_kyc = Arc::new(MockKycProvider::new(
            b"insecure-dev-only-kyc-secret".to_vec(),
        ));
        Self {
            repository: Arc::new(InMemoryRepository::default()),
            ephemeral: Arc::new(InMemoryEphemeralStore::default()),
            rate_limiter: Arc::new(InMemoryRateLimiter::default()),
            google: Arc::new(DisabledGoogleIdentityProvider),
            wallet_proof: Arc::new(SiweWalletProofVerifier),
            kyc_provider: mock_kyc.clone(),
            #[cfg(debug_assertions)]
            sandbox_kyc: Some(mock_kyc),
            portfolio_trading: Arc::new(StubPortfolioTradingService::default()),
            event_producer: Arc::new(messaging::NoopEventProducer),
            metadata_storage: Arc::new(DisabledMetadataStorage),
            mint_setup: Arc::new(DisabledMintSetupExecutor),
            payment_assets: Arc::new(DisabledPaymentAssetRegistry),
            price_feed: Arc::new(SimulatedPriceFeed),
            siwe_domain: "localhost:3000".to_string(),
            public_origin: "http://localhost:3000".to_string(),
            trusted_role_assignments: Vec::new(),
            trusted_email_roles: Vec::new(),
            security_policy: None,
        }
    }
}

impl GatewayStateBuilder {
    pub fn repository(mut self, repository: Arc<dyn Repository>) -> Self {
        self.repository = repository;
        self
    }
    pub fn ephemeral(mut self, ephemeral: Arc<dyn EphemeralStore>) -> Self {
        self.ephemeral = ephemeral;
        self
    }
    pub fn rate_limiter(mut self, rate_limiter: Arc<dyn RateLimiter>) -> Self {
        self.rate_limiter = rate_limiter;
        self
    }
    #[cfg(test)]
    fn google(mut self, google: Arc<dyn GoogleIdentityProvider>) -> Self {
        self.google = google;
        self
    }
    fn wallet_proof(mut self, wallet_proof: Arc<dyn WalletProofVerifier>) -> Self {
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
        self.portfolio_trading = portfolio_trading;
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
    pub fn mint_setup(mut self, mint_setup: Arc<dyn MintSetupExecutor>) -> Self {
        self.mint_setup = mint_setup;
        self
    }
    pub fn payment_assets(mut self, payment_assets: Arc<dyn PaymentAssetRegistry>) -> Self {
        self.payment_assets = payment_assets;
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

    pub fn trusted_role_assignments(mut self, assignments: Vec<TrustedRoleAssignment>) -> Self {
        self.trusted_role_assignments = assignments;
        self
    }

    pub fn trusted_email_roles(mut self, assignments: Vec<(String, UserRole)>) -> Self {
        self.trusted_email_roles = assignments;
        self
    }

    pub fn security_policy(mut self, policy: BrokerageSecurityPolicy) -> Self {
        self.security_policy = Some(policy);
        self
    }

    pub fn build(self) -> GatewayState {
        let auth_service = if let Some(policy) = self.security_policy {
            Arc::new(BrokerageAuthService::new(policy))
        } else {
            Arc::new(BrokerageAuthService::new(BrokerageSecurityPolicy::default()))
        };
        GatewayState {
            auth_service,
            repository: self.repository,
            ephemeral: self.ephemeral,
            rate_limiter: self.rate_limiter,
            google: self.google,
            wallet_proof: self.wallet_proof,
            kyc_provider: self.kyc_provider,
            #[cfg(debug_assertions)]
            sandbox_kyc: self.sandbox_kyc,
            portfolio_trading: self.portfolio_trading,
            event_producer: self.event_producer,
            metadata_storage: self.metadata_storage,
            mint_setup: self.mint_setup,
            payment_assets: self.payment_assets,
            price_feed: self.price_feed,
            siwe_domain: self.siwe_domain.into(),
            public_origin: self.public_origin.into(),
            trusted_role_assignments: Arc::new(self.trusted_role_assignments),
            trusted_email_roles: Arc::new(self.trusted_email_roles),
        }
    }
}

#[derive(Clone)]
pub struct GatewayState {
    auth_service: Arc<BrokerageAuthService>,
    repository: Arc<dyn Repository>,
    ephemeral: Arc<dyn EphemeralStore>,
    rate_limiter: Arc<dyn RateLimiter>,
    google: Arc<dyn GoogleIdentityProvider>,
    wallet_proof: Arc<dyn WalletProofVerifier>,
    kyc_provider: Arc<dyn KycProviderApi>,
    #[cfg(debug_assertions)]
    sandbox_kyc: Option<Arc<MockKycProvider>>,
    portfolio_trading: Arc<dyn PortfolioTradingApi>,
    event_producer: Arc<dyn EventProducer>,
    metadata_storage: Arc<dyn MetadataStorage>,
    mint_setup: Arc<dyn MintSetupExecutor>,
    payment_assets: Arc<dyn PaymentAssetRegistry>,
    price_feed: Arc<dyn PriceFeed>,
    siwe_domain: Arc<str>,
    public_origin: Arc<str>,
    trusted_role_assignments: Arc<Vec<TrustedRoleAssignment>>,
    trusted_email_roles: Arc<Vec<(String, UserRole)>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedRoleAssignment {
    pub issuer: String,
    pub subject: String,
    pub role: UserRole,
}

impl GatewayState {
    fn disabled() -> Self {
        GatewayStateBuilder::default().build()
    }

    #[cfg(test)]
    fn with_providers(
        google: Arc<dyn GoogleIdentityProvider>,
        wallet_proof: Arc<dyn WalletProofVerifier>,
        siwe_domain: String,
        public_origin: String,
    ) -> Self {
        GatewayStateBuilder::default()
            .google(google)
            .wallet_proof(wallet_proof)
            .siwe_domain(siwe_domain)
            .public_origin(public_origin)
            .build()
    }

    async fn issue_session(
        &self,
        user_id: &str,
        mfa_verified_at: Option<u64>,
    ) -> Result<String, AuthError> {
        let token = random_hex(32)?;
        self.ephemeral
            .create_session(
                &hash_token(&token),
                SessionRecord {
                    user_id: user_id.to_string(),
                    mfa_verified_at,
                },
                SESSION_TTL_SECONDS,
            )
            .await?;
        Ok(token)
    }

    /// Both an expired and a never-issued session are indistinguishable
    /// once ephemeral state carries its own TTL (an expired entry simply
    /// reads back as absent) -- this collapses what the original in-memory
    /// version reported as two distinct errors (`SessionExpired` vs
    /// `Unauthorized`) into one, but both already mapped to the same
    /// `401 Unauthorized` HTTP status, so this is not a behavior change for
    /// any caller.
    async fn authenticate(&self, token: &str) -> Result<SessionRecord, AuthError> {
        self.ephemeral
            .get_session(&hash_token(token))
            .await?
            .ok_or(AuthError::Unauthorized)
    }

    async fn session_for_identity(
        &self,
        identity: VerifiedGoogleIdentity,
    ) -> Result<AuthSessionResponse, AuthError> {
        let new_user_id = format!("usr_{}", random_hex(16)?);
        let new_profile = self
            .auth_service
            .profile_from_verified_google(new_user_id.clone(), &identity)?;
        let assigned_role = self
            .trusted_role_assignments
            .iter()
            .find(|assignment| {
                assignment.issuer == identity.issuer() && assignment.subject == identity.subject()
            })
            .map(|assignment| assignment.role);
        let initial_roles = assigned_role
            .map(|role| HashSet::from([role]))
            .unwrap_or_else(|| HashSet::from([UserRole::Trader]));
        let mut permissions = HashSet::from([WalletAction::ViewPortfolio]);
        // Grant TradeSpot permission to users with Trader role
        if initial_roles.contains(&UserRole::Trader) {
            permissions.insert(WalletAction::TradeSpot);
        }
        let new_principal = AuthPrincipal::new(new_user_id.clone(), initial_roles, permissions);
        let (user_id, mut record) = self
            .repository
            .get_or_create_user_by_identity(
                identity.issuer(),
                identity.subject(),
                &new_user_id,
                new_profile,
                new_principal,
            )
            .await?;
        if let Some(role) = assigned_role {
            record.principal.replace_roles(HashSet::from([role]));
            // Also grant TradeSpot permission if the assigned role is Trader
            if role == UserRole::Trader {
                record.principal.grant_permission(WalletAction::TradeSpot);
            }
            self.repository.save_user(record.clone()).await?;
        }
        let status = self.auth_service.onboarding_status(&record.profile);
        let access_token = self.issue_session(&user_id, None).await?;
        Ok(AuthSessionResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: SESSION_TTL_SECONDS,
            profile: record.profile,
            status,
        })
    }

    fn role_for_email(&self, email: &str) -> UserRole {
        self.trusted_email_roles
            .iter()
            .find(|(configured, _)| configured == email)
            .map(|(_, role)| *role)
            .unwrap_or(UserRole::Trader)
    }

    async fn password_session(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthSessionResponse, AuthError> {
        let credential = self
            .repository
            .get_password_credential(email)
            .await?
            .ok_or(AuthError::Unauthorized)?;
        let hash =
            PasswordHash::new(&credential.password_hash).map_err(|_| AuthError::Unauthorized)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .map_err(|_| AuthError::Unauthorized)?;
        let mut record = self
            .repository
            .get_user(&credential.user_id)
            .await?
            .ok_or(AuthError::Unauthorized)?;
        record
            .principal
            .replace_roles(HashSet::from([self.role_for_email(email)]));
        if record.principal.has_role(UserRole::Trader) {
            record.principal.grant_permission(WalletAction::TradeSpot);
        }
        self.repository.save_user(record.clone()).await?;
        let status = self.auth_service.onboarding_status(&record.profile);
        let access_token = self.issue_session(&credential.user_id, None).await?;
        Ok(AuthSessionResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: SESSION_TTL_SECONDS,
            profile: record.profile,
            status,
        })
    }

    /// Reuses the existing wallet-authorization gate (ownership + role +
    /// permission + onboarding + MFA freshness) for portfolio/trading
    /// actions by authorizing against the user's first linked wallet. These
    /// routes are not inherently wallet-specific, but this keeps a single,
    /// already-audited authorization path in front of every financial
    /// action instead of a second parallel one.
    async fn authorize_trading_action(
        &self,
        user: &AuthenticatedUser,
        action: WalletAction,
    ) -> Result<UserRecord, ApiError> {
        let record = self
            .repository
            .get_user(&user.user_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
        let wallet_id = record
            .profile
            .wallet_ids()
            .first()
            .ok_or_else(|| ApiError::from(AuthError::MissingWallet))?;
        let wallet = self
            .repository
            .get_wallet(wallet_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::from(AuthError::MissingWallet))?;
        self.auth_service
            .authorize_wallet_action(
                &record.principal,
                &record.profile,
                &wallet,
                action,
                user.mfa_verified_at,
                unix_now(),
            )
            .map_err(ApiError::from)?;
        Ok(record)
    }
}

#[derive(Debug, Clone)]
struct AuthenticatedUser {
    user_id: String,
    mfa_verified_at: Option<u64>,
}

#[async_trait]
impl FromRequestParts<GatewayState> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayState,
    ) -> Result<Self, Self::Rejection> {
        let token =
            bearer_token(&parts.headers).ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
        let session = state.authenticate(token).await.map_err(ApiError::from)?;
        Ok(Self {
            user_id: session.user_id,
            mfa_verified_at: session.mfa_verified_at,
        })
    }
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    error: AuthError,
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        let status = match error {
            AuthError::Unauthorized | AuthError::SessionExpired => StatusCode::UNAUTHORIZED,
            AuthError::ForbiddenAction
            | AuthError::WalletNotOwned
            | AuthError::OnboardingIncomplete
            | AuthError::MfaRequired => StatusCode::FORBIDDEN,
            AuthError::ReplayDetected => StatusCode::CONFLICT,
            AuthError::InvalidProvider => StatusCode::SERVICE_UNAVAILABLE,
            AuthError::InvalidIdentityProof
            | AuthError::UnverifiedEmail
            | AuthError::InvalidWalletProof => StatusCode::UNAUTHORIZED,
            AuthError::ChallengeExpired => StatusCode::GONE,
            AuthError::MissingWallet => StatusCode::NOT_FOUND,
            AuthError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            AuthError::PolicyViolation(_) => StatusCode::BAD_REQUEST,
        };
        Self { status, error }
    }
}

impl From<KycError> for ApiError {
    fn from(error: KycError) -> Self {
        let status = match error {
            KycError::CaseNotFound => StatusCode::NOT_FOUND,
            KycError::InvalidSignature | KycError::MalformedPayload => StatusCode::BAD_REQUEST,
            KycError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        Self {
            status,
            error: AuthError::PolicyViolation(error.to_string()),
        }
    }
}

impl From<portfolio_trading::TradingError> for ApiError {
    fn from(error: portfolio_trading::TradingError) -> Self {
        let status = match error {
            portfolio_trading::TradingError::OrderNotFound => StatusCode::NOT_FOUND,
            portfolio_trading::TradingError::InvalidQuantity(_) => StatusCode::BAD_REQUEST,
            portfolio_trading::TradingError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        Self {
            status,
            error: AuthError::PolicyViolation(error.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(ApiErrorResponse {
                code: self.error.code().to_string(),
                message: self.error.to_string(),
            }),
        )
            .into_response()
    }
}

#[cfg(debug_assertions)]
#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        google_authorization_handler,
        google_callback_handler,
        password_register_handler,
        password_login_handler,
        auth_me_handler,
        logout_handler,
        issuer_overview_handler,
        issuer_onboarding_handler,
        create_asset_draft_handler,
        list_asset_drafts_handler,
        get_asset_draft_handler,
        update_asset_draft_handler,
        delete_asset_draft_handler,
        submit_asset_for_demo_review_handler,
        start_asset_setup_handler,
        get_asset_setup_operation_handler,
        get_initial_inventory_handler,
        issue_initial_inventory_handler,
        submit_asset_backing_handler,
        publish_asset_listing_handler,
        cngn_payment_asset_handler,
        create_broker_cngn_account_handler,
        onboarding_status_handler,
        accept_terms_handler,
        wallet_challenge_handler,
        solana_wallet_challenge_handler,
        wallet_link_handler,
        list_wallets_handler,
        authorize_wallet_handler,
        investor_cngn_account_handler,
        create_investor_cngn_account_handler,
        investor_catalog_handler,
        prepare_investor_purchase_handler,
        submit_investor_purchase_handler
    ),
    components(schemas(
        HealthResponse,
        ApiErrorResponse,
        GoogleAuthorizationResponse,
        GoogleCallbackRequest,
        PasswordRegisterRequest,
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
struct ApiDoc;

pub fn router() -> Router {
    router_with_state(GatewayState::disabled())
}

/// Fail-closed router used when provider/bootstrap setup fails. It preserves
/// the configured browser origin so the frontend receives the actual auth
/// error instead of a misleading CORS failure.
pub fn fail_closed_router(public_origin: impl Into<String>) -> Router {
    router_with_state(
        GatewayStateBuilder::default()
            .public_origin(public_origin)
            .build(),
    )
}

async fn ip_rate_limit_middleware(
    State(state): State<GatewayState>,
    request: Request,
    next: Next,
) -> Result<axum::response::Response, ApiError> {
    // Trustworthy only because the WAF container in front of this service
    // (see the `waf` docker-compose service) is the sole path a real client
    // request can take to reach it and is what sets this header -- the
    // gateway is not exposed directly. A raw socket peer address is not
    // used here since axum's `ConnectInfo` reflects the WAF's own address,
    // not the original client's, once the WAF is in place.
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .unwrap_or("unknown")
        .to_string();
    state
        .rate_limiter
        .check(&format!("ip:{ip}"), IP_RATE_LIMIT.0, IP_RATE_LIMIT.1)
        .await
        .map_err(ApiError::from)?;
    Ok(next.run(request).await)
}

fn router_with_state(state: GatewayState) -> Router {
    http::router::build(state)
}

// `kyc_sandbox_advance_handler` only exists at all under `#[cfg(debug_assertions)]`
// (release builds disable debug_assertions), so this dispatch must happen at
// compile time via `#[cfg(...)]` -- a runtime `if cfg!(debug_assertions)` would
// still compile both branches and fail to link in release mode.
#[cfg(debug_assertions)]
fn add_debug_routes(router: Router<GatewayState>) -> Router<GatewayState> {
    router
        .route("/kyc/_sandbox/advance", post(kyc_sandbox_advance_handler))
        .route("/swagger-ui", get(swagger_ui_handler))
        .route("/api-docs/openapi.json", get(openapi_handler))
}

#[cfg(not(debug_assertions))]
fn add_debug_routes(router: Router<GatewayState>) -> Router<GatewayState> {
    router
}

#[cfg(debug_assertions)]
async fn openapi_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[cfg(debug_assertions)]
async fn swagger_ui_handler() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>Rabovel API</title>
<link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head>
<body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>SwaggerUIBundle({url:'/api-docs/openapi.json',dom_id:'#swagger-ui'});</script></body></html>"#,
    )
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = HealthResponse)))]
async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/auth/google/start",
    responses((status = 200, body = GoogleAuthorizationResponse), (status = 503, body = ApiErrorResponse))
)]
async fn google_authorization_handler(
    State(state): State<GatewayState>,
) -> Result<Json<GoogleAuthorizationResponse>, ApiError> {
    let start = state.google.begin().map_err(ApiError::from)?;
    state
        .ephemeral
        .put_oidc_pending(
            &start.state,
            OidcPendingRecord {
                nonce: start.pending.nonce.clone(),
                pkce_verifier: start.pending.pkce_verifier.clone(),
            },
            OIDC_CHALLENGE_TTL_SECONDS,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(GoogleAuthorizationResponse {
        authorization_url: start.authorization_url,
        state: start.state,
        expires_in: OIDC_CHALLENGE_TTL_SECONDS,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/google/callback",
    request_body = GoogleCallbackRequest,
    responses((status = 200, body = AuthSessionResponse), (status = 401, body = ApiErrorResponse))
)]
async fn google_callback_handler(
    State(state): State<GatewayState>,
    Json(request): Json<GoogleCallbackRequest>,
) -> Result<Json<AuthSessionResponse>, ApiError> {
    if state
        .ephemeral
        .is_oidc_state_used(&request.state)
        .await
        .map_err(ApiError::from)?
    {
        return Err(ApiError::from(AuthError::ReplayDetected));
    }
    let pending_record = state
        .ephemeral
        .take_oidc_pending(&request.state)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::InvalidIdentityProof))?;
    state
        .ephemeral
        .mark_oidc_state_used(&request.state, OIDC_CHALLENGE_TTL_SECONDS)
        .await
        .map_err(ApiError::from)?;
    let pending = OidcPending {
        nonce: pending_record.nonce,
        pkce_verifier: pending_record.pkce_verifier,
    };
    let identity = state
        .google
        .exchange(&request.code, &pending)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        state
            .session_for_identity(identity)
            .await
            .map_err(ApiError::from)?,
    ))
}

#[utoipa::path(post, path = "/auth/register", request_body = PasswordRegisterRequest, responses((status = 200, body = AuthSessionResponse), (status = 400, body = ApiErrorResponse)))]
async fn password_register_handler(
    State(state): State<GatewayState>,
    Json(request): Json<PasswordRegisterRequest>,
) -> Result<Json<AuthSessionResponse>, ApiError> {
    let email = request.email.trim().to_ascii_lowercase();
    let display_name = request.display_name.trim().to_string();
    if !email.contains('@') || display_name.len() < 2 || request.password.len() < 8 {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "a valid name, email, and password of at least 8 characters are required".to_string(),
        )));
    }
    let user_id = format!("usr_{}", random_hex(16).map_err(ApiError::from)?);
    let mut profile = state
        .auth_service
        .profile_from_email_password(user_id.clone(), email.clone())
        .map_err(ApiError::from)?;
    profile.accept_terms("demo-v1").map_err(ApiError::from)?;
    let role = state.role_for_email(&email);
    let principal = AuthPrincipal::new(
        user_id.clone(),
        HashSet::from([role]),
        HashSet::from([WalletAction::ViewPortfolio]),
    );
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(|_| {
            ApiError::from(AuthError::PolicyViolation(
                "password hashing failed".to_string(),
            ))
        })?
        .to_string();
    let record = state
        .repository
        .create_password_user(
            PasswordCredential {
                user_id: user_id.clone(),
                email,
                display_name,
                password_hash,
            },
            profile,
            principal,
        )
        .await
        .map_err(ApiError::from)?;
    let status = state.auth_service.onboarding_status(&record.profile);
    let access_token = state
        .issue_session(&user_id, None)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(AuthSessionResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: SESSION_TTL_SECONDS,
        profile: record.profile,
        status,
    }))
}

#[utoipa::path(post, path = "/auth/login", request_body = PasswordLoginRequest, responses((status = 200, body = AuthSessionResponse), (status = 401, body = ApiErrorResponse)))]
async fn password_login_handler(
    State(state): State<GatewayState>,
    Json(request): Json<PasswordLoginRequest>,
) -> Result<Json<AuthSessionResponse>, ApiError> {
    let email = request.email.trim().to_ascii_lowercase();
    Ok(Json(
        state
            .password_session(&email, &request.password)
            .await
            .map_err(ApiError::from)?,
    ))
}

#[utoipa::path(get, path = "/auth/me", responses((status = 200, body = CurrentUserResponse), (status = 401, body = ApiErrorResponse)))]
async fn auth_me_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<CurrentUserResponse>, ApiError> {
    let record = state
        .repository
        .get_user(&user.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    let email = record.profile.email().to_string();
    let display_name = state
        .repository
        .get_password_credential_by_user_id(&user.user_id)
        .await
        .map_err(ApiError::from)?
        .map(|credential| credential.display_name)
        .unwrap_or_else(|| email.split('@').next().unwrap_or(&email).to_string());
    let mut roles: Vec<_> = record.principal.roles_iter().collect();
    roles.sort_by_key(|role| match role {
        UserRole::Admin => 0,
        UserRole::ComplianceOfficer => 1,
        UserRole::RiskOps => 2,
        UserRole::Issuer => 3,
        UserRole::Trader => 4,
    });
    Ok(Json(CurrentUserResponse {
        user_id: user.user_id,
        email,
        display_name,
        roles,
        onboarding_status: state.auth_service.onboarding_status(&record.profile),
    }))
}

#[utoipa::path(post, path = "/auth/logout", responses((status = 204), (status = 401, body = ApiErrorResponse)))]
async fn logout_handler(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    _user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let token = bearer_token(&headers).ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    state
        .ephemeral
        .delete_session(&hash_token(token))
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/issuer/overview", responses((status = 200, body = IssuerOverviewResponse), (status = 401, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse)))]
async fn issuer_overview_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<IssuerOverviewResponse>, ApiError> {
    let record = state
        .repository
        .get_user(&user.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    if !record.principal.has_role(UserRole::Issuer) {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }
    let organization = state
        .repository
        .get_issuer_organization(&user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(IssuerOverviewResponse {
        user_id: user.user_id,
        email: record.profile.email().to_string(),
        account_status: if organization.is_some() {
            "approved"
        } else {
            "organization_onboarding_pending"
        }
        .to_string(),
        organization: organization.map(issuer_organization_response),
    }))
}

fn issuer_organization_response(organization: IssuerOrganization) -> IssuerOrganizationResponse {
    IssuerOrganizationResponse {
        legal_name: organization.legal_name,
        organization_type: organization.organization_type,
        registration_number: organization.registration_number,
        jurisdiction: organization.jurisdiction,
        registered_address: organization.registered_address,
        representative_name: organization.representative_name,
        representative_title: organization.representative_title,
        document_reference: organization.document_reference,
        approved_at: organization.approved_at,
    }
}

#[utoipa::path(post, path = "/issuer/onboarding", request_body = IssuerOnboardingRequest, responses((status = 200, body = IssuerOverviewResponse), (status = 400, body = ApiErrorResponse), (status = 401, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse)))]
async fn issuer_onboarding_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<IssuerOnboardingRequest>,
) -> Result<Json<IssuerOverviewResponse>, ApiError> {
    let record = state
        .repository
        .get_user(&user.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    if !record.principal.has_role(UserRole::Issuer) {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }

    let fields = [
        &request.legal_name,
        &request.organization_type,
        &request.registration_number,
        &request.jurisdiction,
        &request.registered_address,
        &request.representative_name,
        &request.representative_title,
        &request.document_reference,
    ];
    if fields.iter().any(|value| value.trim().len() < 2)
        || !request.beneficial_owners_confirmed
        || !request.information_certified
    {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "complete every organization field and confirm both attestations".to_string(),
        )));
    }

    let organization = IssuerOrganization {
        user_id: user.user_id.clone(),
        legal_name: request.legal_name.trim().to_string(),
        organization_type: request.organization_type.trim().to_string(),
        registration_number: request.registration_number.trim().to_string(),
        jurisdiction: request.jurisdiction.trim().to_string(),
        registered_address: request.registered_address.trim().to_string(),
        representative_name: request.representative_name.trim().to_string(),
        representative_title: request.representative_title.trim().to_string(),
        document_reference: request.document_reference.trim().to_string(),
        approved_at: unix_now(),
    };
    state
        .repository
        .save_issuer_organization(organization.clone())
        .await
        .map_err(ApiError::from)?;

    Ok(Json(IssuerOverviewResponse {
        user_id: user.user_id,
        email: record.profile.email().to_string(),
        account_status: "approved".to_string(),
        organization: Some(issuer_organization_response(organization)),
    }))
}

async fn require_approved_issuer(state: &GatewayState, user_id: &str) -> Result<(), ApiError> {
    let record = state
        .repository
        .get_user(user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    if !record.principal.has_role(UserRole::Issuer) {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }
    if state
        .repository
        .get_issuer_organization(user_id)
        .await
        .map_err(ApiError::from)?
        .is_none()
    {
        return Err(ApiError::from(AuthError::OnboardingIncomplete));
    }
    Ok(())
}

fn asset_draft_response(record: AssetDraftRecord) -> AssetDraftResponse {
    AssetDraftResponse {
        asset_id: record.asset_id,
        issuer_user_id: record.issuer_user_id,
        status: record.status,
        issued_units: "0".into(),
        mint_address: record.mint_address,
        created_at: record.created_at,
        updated_at: record.updated_at,
        draft: record.draft,
    }
}

#[utoipa::path(post, path = "/issuer/assets", request_body = CreateAssetDraftRequest, responses((status = 200, body = AssetDraftResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse)))]
async fn create_asset_draft_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateAssetDraftRequest>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    let draft = asset_draft_from_request(request)?;
    let now = unix_now();
    let record = AssetDraftRecord {
        asset_id: format!("ast_{}", random_hex(12).map_err(ApiError::from)?),
        issuer_user_id: user.user_id,
        canonical_key: draft.canonical_key(),
        draft,
        status: "draft".into(),
        created_at: now,
        updated_at: now,
        mint_address: None,
    };
    state
        .repository
        .create_asset_draft(record.clone())
        .await
        .map_err(ApiError::from)?;
    Ok(Json(asset_draft_response(record)))
}

fn asset_draft_from_request(request: CreateAssetDraftRequest) -> Result<NewAssetDraft, ApiError> {
    if request.metadata.metadata_uri.is_some() {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "metadata_uri is assigned by Rabovel after publishing".into(),
        )));
    }
    let authorized_units = request.authorized_units.parse::<u64>().map_err(|_| {
        ApiError::from(AuthError::PolicyViolation(
            "authorized_units must be a positive whole-number string".into(),
        ))
    })?;
    NewAssetDraft {
        instrument_code: request.instrument_code,
        name: request.name,
        ticker: request.ticker,
        market: request.market,
        share_class: request.share_class,
        asset_type: request.asset_type,
        decimals: request.decimals,
        authorized_units,
        settlement_currency: request.settlement_currency,
        representation: request.representation,
        rights_description: request.rights_description,
        disclosure: request.disclosure,
        metadata: request.metadata,
        backing: None,
        listing_status: "not_listed".into(),
    }
    .validate_and_normalize()
    .map_err(|message| ApiError::from(AuthError::PolicyViolation(message)))
}

#[utoipa::path(get, path = "/issuer/assets", responses((status = 200, body = [AssetDraftResponse]), (status = 403, body = ApiErrorResponse)))]
async fn list_asset_drafts_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<AssetDraftResponse>>, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    let assets = state
        .repository
        .list_asset_drafts(&user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(assets.into_iter().map(asset_draft_response).collect()))
}

#[utoipa::path(get, path = "/issuer/assets/{asset_id}", responses((status = 200, body = AssetDraftResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
async fn get_asset_draft_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    let asset = state
        .repository
        .get_asset_draft(&asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            error: AuthError::PolicyViolation("asset draft not found".into()),
        })?;
    if asset.issuer_user_id != user.user_id {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }
    Ok(Json(asset_draft_response(asset)))
}

const MAX_ASSET_IMAGE_BYTES: u64 = 5 * 1024 * 1024;

async fn create_asset_image_upload_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
    Json(request): Json<ImageUploadRequest>,
) -> Result<Json<ImageUploadTicket>, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    let asset = state
        .repository
        .get_asset_draft(&asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            error: AuthError::PolicyViolation("asset draft not found".into()),
        })?;
    if asset.issuer_user_id != user.user_id || asset.status != "draft" {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }
    if request.size == 0 || request.size > MAX_ASSET_IMAGE_BYTES {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "image must be between 1 byte and 5 MB".into(),
        )));
    }
    let extension = match request.content_type.as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => {
            return Err(ApiError::from(AuthError::PolicyViolation(
                "image must be PNG, JPEG, or WebP".into(),
            )))
        }
    };
    let object_path = format!(
        "assets/{asset_id}/images/{}.{}",
        random_hex(16).map_err(ApiError::from)?,
        extension
    );
    let signed_url = state
        .metadata_storage
        .create_signed_image_upload(&object_path)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ImageUploadTicket {
        signed_url,
        object_path,
    }))
}

async fn confirm_asset_image_upload_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
    Json(request): Json<ConfirmImageUploadRequest>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    let expected_prefix = format!("assets/{asset_id}/images/");
    if !request.object_path.starts_with(&expected_prefix) {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }
    if !matches!(
        request.content_type.as_str(),
        "image/png" | "image/jpeg" | "image/webp"
    ) {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "image must be PNG, JPEG, or WebP".into(),
        )));
    }
    let image_uri = state
        .metadata_storage
        .confirm_public_image(
            &request.object_path,
            &request.content_type,
            MAX_ASSET_IMAGE_BYTES,
        )
        .await
        .map_err(ApiError::from)?;
    let asset = state
        .repository
        .set_asset_image_uri(&asset_id, &user.user_id, image_uri, unix_now())
        .await
        .map_err(ApiError::from)?;
    Ok(Json(asset_draft_response(asset)))
}

#[utoipa::path(patch, path = "/issuer/assets/{asset_id}", request_body = CreateAssetDraftRequest, responses((status = 200, body = AssetDraftResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
async fn update_asset_draft_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
    Json(request): Json<CreateAssetDraftRequest>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    let existing = state
        .repository
        .get_asset_draft(&asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            error: AuthError::PolicyViolation("asset draft not found".into()),
        })?;
    if existing.issuer_user_id != user.user_id || existing.status != "draft" {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }
    let draft = asset_draft_from_request(request)?;
    let updated = AssetDraftRecord {
        canonical_key: draft.canonical_key(),
        draft,
        updated_at: unix_now(),
        ..existing
    };
    state
        .repository
        .update_asset_draft(updated.clone())
        .await
        .map_err(ApiError::from)?;
    Ok(Json(asset_draft_response(updated)))
}

#[utoipa::path(delete, path = "/issuer/assets/{asset_id}", responses((status = 204), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
async fn delete_asset_draft_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    state
        .repository
        .delete_asset_draft(&asset_id, &user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/issuer/assets/{asset_id}/submit", responses((status = 200, body = AssetDraftResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
async fn submit_asset_for_demo_review_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    let asset = state
        .repository
        .get_asset_draft(&asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            error: AuthError::PolicyViolation("asset draft not found".into()),
        })?;
    if asset.issuer_user_id != user.user_id {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }
    if asset.draft.metadata.metadata_uri.is_none() {
        let organization = state
            .repository
            .get_issuer_organization(&user.user_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::from(AuthError::OnboardingIncomplete))?;
        let document = asset_metadata_document(&asset, &organization);
        let metadata_uri = state
            .metadata_storage
            .publish(&asset.asset_id, &document)
            .await
            .map_err(ApiError::from)?;
        state
            .repository
            .set_asset_metadata_uri(&asset.asset_id, &user.user_id, metadata_uri, unix_now())
            .await
            .map_err(ApiError::from)?;
    }
    let approved = state
        .repository
        .approve_asset_for_demo_setup(&asset_id, &user.user_id, unix_now())
        .await
        .map_err(ApiError::from)?;
    Ok(Json(asset_draft_response(approved)))
}

fn asset_setup_response(operation: AssetSetupOperation) -> AssetSetupOperationResponse {
    AssetSetupOperationResponse {
        operation_id: operation.operation_id,
        asset_id: operation.asset_id,
        network: operation.network,
        mint_address: operation.mint_address,
        status: operation.status,
        stages: operation
            .stages
            .into_iter()
            .map(|stage| SetupStageResponse {
                stage: stage.stage,
                status: stage.status,
                signatures: stage.signatures,
                verified_at: stage.verified_at,
            })
            .collect(),
        mint_config_address: operation.mint_config_address,
        allow_list_address: operation.allow_list_address,
        block_list_address: operation.block_list_address,
        thaw_extra_metas_address: operation.thaw_extra_metas_address,
        error: operation.error,
        created_at: operation.created_at,
        updated_at: operation.updated_at,
    }
}

fn initial_inventory_response(inventory: InitialInventory) -> InitialInventoryResponse {
    InitialInventoryResponse {
        network: inventory.network,
        mint_address: inventory.mint_address,
        settlement_wallet: inventory.settlement_wallet,
        token_account: inventory.token_account,
        authorized_units: inventory.authorized_units,
        supply: inventory.supply,
        inventory_balance: inventory.inventory_balance,
        wallet_allowlisted: inventory.wallet_allowlisted,
        token_account_ready: inventory.token_account_ready,
        issuance_complete: inventory.issuance_complete,
        signatures: inventory.signatures,
    }
}

async fn inventory_context(
    state: &GatewayState,
    user_id: &str,
    asset_id: &str,
) -> Result<(AssetDraftRecord, AssetSetupOperation), ApiError> {
    let asset = owned_setup_asset(state, user_id, asset_id).await?;
    let operation = state
        .repository
        .get_asset_setup_operation(asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::CONFLICT,
            error: AuthError::PolicyViolation("asset mint setup has not started".into()),
        })?;
    if operation.status != "confirmed" {
        return Err(ApiError {
            status: StatusCode::CONFLICT,
            error: AuthError::PolicyViolation("asset mint setup is not confirmed".into()),
        });
    }
    Ok((asset, operation))
}

#[utoipa::path(get, path = "/issuer/assets/{asset_id}/inventory", responses((status = 200, body = InitialInventoryResponse), (status = 403, body = ApiErrorResponse), (status = 409, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
async fn get_initial_inventory_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<InitialInventoryResponse>, ApiError> {
    let (asset, operation) = inventory_context(&state, &user.user_id, &asset_id).await?;
    state
        .mint_setup
        .initial_inventory(&asset, &operation, false)
        .await
        .map(initial_inventory_response)
        .map(Json)
        .map_err(|error| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(error.message),
        })
}

#[utoipa::path(post, path = "/issuer/assets/{asset_id}/inventory", responses((status = 200, body = InitialInventoryResponse), (status = 403, body = ApiErrorResponse), (status = 409, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
async fn issue_initial_inventory_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<InitialInventoryResponse>, ApiError> {
    let (asset, operation) = inventory_context(&state, &user.user_id, &asset_id).await?;
    state
        .mint_setup
        .initial_inventory(&asset, &operation, true)
        .await
        .map(initial_inventory_response)
        .map(Json)
        .map_err(|error| ApiError {
            status: if error.reconciliation_required {
                StatusCode::CONFLICT
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            },
            error: AuthError::PolicyViolation(error.message),
        })
}

#[utoipa::path(post, path = "/issuer/assets/{asset_id}/backing", request_body = SubmitBackingRequest, responses((status = 200, body = AssetDraftResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse)))]
async fn submit_asset_backing_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
    Json(request): Json<SubmitBackingRequest>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    let summary = request.summary.trim();
    let document_name = request.document_name.trim();
    if summary.len() < 10 || summary.len() > 500 {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "backing summary must be between 10 and 500 characters".into(),
        )));
    }
    if document_name.is_empty() || document_name.len() > 160 {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "backing document name is required and must be at most 160 characters".into(),
        )));
    }
    if !matches!(
        request.content_type.as_str(),
        "application/pdf" | "image/png" | "image/jpeg"
    ) {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "backing document must be a PDF, PNG, or JPEG".into(),
        )));
    }
    if request.size_bytes == 0 || request.size_bytes > 10 * 1024 * 1024 {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "backing document must be between 1 byte and 10 MB".into(),
        )));
    }
    let now = unix_now();
    let record = state
        .repository
        .save_asset_backing(
            &asset_id,
            &user.user_id,
            BackingEvidence {
                summary: summary.into(),
                document_name: document_name.into(),
                content_type: request.content_type,
                size_bytes: request.size_bytes,
                verification_status: "verified".into(),
                verified_at: now,
            },
            now,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(asset_draft_response(record)))
}

#[utoipa::path(post, path = "/issuer/assets/{asset_id}/listing", responses((status = 200, body = AssetDraftResponse), (status = 403, body = ApiErrorResponse), (status = 409, body = ApiErrorResponse)))]
async fn publish_asset_listing_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetDraftResponse>, ApiError> {
    let (asset, operation) = inventory_context(&state, &user.user_id, &asset_id).await?;
    if asset
        .draft
        .backing
        .as_ref()
        .is_none_or(|backing| backing.verification_status != "verified")
    {
        return Err(ApiError {
            status: StatusCode::CONFLICT,
            error: AuthError::PolicyViolation("verified backing is required before listing".into()),
        });
    }
    let inventory = state
        .mint_setup
        .initial_inventory(&asset, &operation, false)
        .await
        .map_err(|error| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(error.message),
        })?;
    if !inventory.issuance_complete {
        return Err(ApiError {
            status: StatusCode::CONFLICT,
            error: AuthError::PolicyViolation(
                "initial inventory must be issued before listing".into(),
            ),
        });
    }
    let record = state
        .repository
        .publish_asset_listing(&asset_id, &user.user_id, unix_now())
        .await
        .map_err(ApiError::from)?;
    Ok(Json(asset_draft_response(record)))
}

#[utoipa::path(get, path = "/payment-assets/cngn", responses((status = 200, body = PaymentAssetStatus), (status = 401, body = ApiErrorResponse)))]
async fn cngn_payment_asset_handler(
    State(state): State<GatewayState>,
    _user: AuthenticatedUser,
) -> Json<PaymentAssetStatus> {
    Json(state.payment_assets.cngn_status().await)
}

#[utoipa::path(post, path = "/payment-assets/cngn", responses((status = 200, body = PaymentAssetStatus), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
async fn create_broker_cngn_account_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<PaymentAssetStatus>, ApiError> {
    require_approved_issuer(&state, &user.user_id).await?;
    state
        .payment_assets
        .ensure_cngn_broker_account()
        .await
        .map(Json)
        .map_err(|message| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(message),
        })
}

async fn owned_setup_asset(
    state: &GatewayState,
    user_id: &str,
    asset_id: &str,
) -> Result<AssetDraftRecord, ApiError> {
    require_approved_issuer(state, user_id).await?;
    let asset = state
        .repository
        .get_asset_draft(asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            error: AuthError::PolicyViolation("asset draft not found".into()),
        })?;
    if asset.issuer_user_id != user_id {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }
    Ok(asset)
}

#[utoipa::path(get, path = "/issuer/assets/{asset_id}/setup", responses((status = 200, body = AssetSetupOperationResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
async fn get_asset_setup_operation_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetSetupOperationResponse>, ApiError> {
    owned_setup_asset(&state, &user.user_id, &asset_id).await?;
    let operation = state
        .repository
        .get_asset_setup_operation(&asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            error: AuthError::PolicyViolation("asset setup operation not found".into()),
        })?;
    Ok(Json(asset_setup_response(operation)))
}

#[utoipa::path(post, path = "/issuer/assets/{asset_id}/setup", responses((status = 200, body = AssetSetupOperationResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse)))]
async fn start_asset_setup_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetSetupOperationResponse>, ApiError> {
    let asset = owned_setup_asset(&state, &user.user_id, &asset_id).await?;
    if !matches!(asset.status.as_str(), "approved_for_setup" | "minted") {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "asset must be approved before mint setup".into(),
        )));
    }
    if asset.draft.metadata.metadata_uri.is_none() {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "hosted metadata is required before mint setup".into(),
        )));
    }
    if !state.mint_setup.is_configured() {
        return Err(ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation("Solana mint setup is not configured".into()),
        });
    }
    let existing = state
        .repository
        .get_asset_setup_operation(&asset_id)
        .await
        .map_err(ApiError::from)?;
    let mut operation = if let Some(mut existing) = existing {
        if existing.status != "failed" {
            return Ok(Json(asset_setup_response(existing)));
        }
        existing.status = "running".into();
        existing.error = None;
        existing.updated_at = unix_now();
        state
            .repository
            .save_asset_setup_operation(existing.clone())
            .await
            .map_err(ApiError::from)?;
        existing
    } else {
        let now = unix_now();
        let operation_id = format!("setup_{}", random_hex(12).map_err(ApiError::from)?);
        let proposed = AssetSetupOperation {
            operation_id: operation_id.clone(),
            asset_id: asset.asset_id.clone(),
            issuer_user_id: user.user_id.clone(),
            network: state.mint_setup.network().to_string(),
            mint_address: state.mint_setup.mint_address(&operation_id),
            status: "running".into(),
            stages: vec![SetupStageRecord {
                stage: "metadata_ready".into(),
                status: "verified".into(),
                signatures: vec![],
                verified_at: Some(now),
            }],
            mint_config_address: None,
            allow_list_address: None,
            block_list_address: None,
            thaw_extra_metas_address: None,
            error: None,
            created_at: now,
            updated_at: now,
        };
        let created = state
            .repository
            .create_asset_setup_operation(proposed)
            .await
            .map_err(ApiError::from)?;
        if created.operation_id != operation_id {
            return Ok(Json(asset_setup_response(created)));
        }
        created
    };

    match state.mint_setup.execute(&asset, &operation).await {
        Ok(execution) => {
            let verified_at = unix_now();
            operation.status = "confirmed".into();
            operation.updated_at = verified_at;
            operation.mint_config_address = Some(execution.mint_config_address);
            operation.allow_list_address = Some(execution.allow_list_address);
            operation.block_list_address = Some(execution.block_list_address);
            operation.thaw_extra_metas_address = Some(execution.thaw_extra_metas_address);
            operation.stages.extend([
                SetupStageRecord {
                    stage: "mint_verified".into(),
                    status: "verified".into(),
                    signatures: execution.mint_signature.into_iter().collect(),
                    verified_at: Some(verified_at),
                },
                SetupStageRecord {
                    stage: "shared_lists_verified".into(),
                    status: "verified".into(),
                    signatures: execution.shared_lists_signature.into_iter().collect(),
                    verified_at: Some(verified_at),
                },
                SetupStageRecord {
                    stage: "acl_verified".into(),
                    status: "verified".into(),
                    signatures: execution.acl_signatures,
                    verified_at: Some(verified_at),
                },
            ]);
            state
                .repository
                .save_asset_setup_operation(operation.clone())
                .await
                .map_err(ApiError::from)?;
            state
                .repository
                .mark_asset_minted(
                    &asset_id,
                    &user.user_id,
                    &operation.mint_address,
                    verified_at,
                )
                .await
                .map_err(ApiError::from)?;
        }
        Err(error) => {
            operation.status = if error.reconciliation_required {
                "reconciliation_required"
            } else {
                "failed"
            }
            .into();
            operation.error = Some(error.message);
            operation.updated_at = unix_now();
            state
                .repository
                .save_asset_setup_operation(operation.clone())
                .await
                .map_err(ApiError::from)?;
        }
    }
    Ok(Json(asset_setup_response(operation)))
}

fn asset_metadata_document(
    asset: &AssetDraftRecord,
    organization: &IssuerOrganization,
) -> serde_json::Value {
    let draft = &asset.draft;
    let mut properties = serde_json::Map::new();
    for property in &draft.metadata.additional_metadata {
        if matches!(
            property.key.trim().to_ascii_lowercase().as_str(),
            "underlying" | "underlying_reference"
        ) {
            continue;
        }
        properties.insert(
            property.key.clone(),
            serde_json::Value::String(property.value.clone()),
        );
    }
    properties.insert("ratio".into(), serde_json::json!(draft.representation));
    properties.insert("asset_id".into(), serde_json::json!(asset.asset_id));
    properties.insert("asset_type".into(), serde_json::json!(draft.asset_type));
    properties.insert("market".into(), serde_json::json!(draft.market));
    properties.insert("share_class".into(), serde_json::json!(draft.share_class));
    properties.insert(
        "representation".into(),
        serde_json::json!(draft.representation),
    );
    properties.insert(
        "rights_description".into(),
        serde_json::json!(draft.rights_description),
    );
    properties.insert("decimals".into(), serde_json::json!(draft.decimals));
    properties.insert(
        "authorized_units".into(),
        serde_json::json!(draft.authorized_units.to_string()),
    );
    properties.insert(
        "settlement_currency".into(),
        serde_json::json!(draft.settlement_currency),
    );
    properties.insert(
        "issuer_name".into(),
        serde_json::json!(organization.legal_name),
    );
    properties.insert("disclosure".into(), serde_json::json!(draft.disclosure));
    properties.insert("schema_version".into(), serde_json::json!("1.0"));
    let mut document = serde_json::Map::new();
    document.insert("name".into(), serde_json::json!(draft.metadata.name));
    document.insert("symbol".into(), serde_json::json!(draft.metadata.symbol));
    document.insert(
        "description".into(),
        serde_json::json!(draft.metadata.description),
    );
    if let Some(image) = &draft.metadata.image_uri {
        document.insert("image".into(), serde_json::json!(image));
    }
    if let Some(external_url) = &draft.metadata.external_url {
        document.insert("external_url".into(), serde_json::json!(external_url));
    }
    document.insert("properties".into(), serde_json::Value::Object(properties));
    serde_json::Value::Object(document)
}

#[utoipa::path(
    get,
    path = "/onboarding/status",
    responses((status = 200, body = OnboardingStatusResponse), (status = 401, body = ApiErrorResponse))
)]
async fn onboarding_status_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<OnboardingStatusResponse>, ApiError> {
    let record = state
        .repository
        .get_user(&user.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    Ok(Json(OnboardingStatusResponse {
        status: state.auth_service.onboarding_status(&record.profile),
        profile: record.profile,
    }))
}

#[utoipa::path(
    post,
    path = "/onboarding/terms",
    request_body = TermsAcceptanceRequest,
    responses((status = 200, body = OnboardingStatusResponse), (status = 401, body = ApiErrorResponse))
)]
async fn accept_terms_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<TermsAcceptanceRequest>,
) -> Result<Json<OnboardingStatusResponse>, ApiError> {
    let mut record = state
        .repository
        .get_user(&user.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    let before_status = state.auth_service.onboarding_status(&record.profile);
    record
        .profile
        .accept_terms(request.version)
        .map_err(ApiError::from)?;
    state
        .repository
        .save_user(record.clone())
        .await
        .map_err(ApiError::from)?;
    let after_status = state.auth_service.onboarding_status(&record.profile);
    publish_onboarding_change(&state, &user.user_id, before_status, after_status).await;
    Ok(Json(OnboardingStatusResponse {
        status: after_status,
        profile: record.profile,
    }))
}

#[utoipa::path(
    post,
    path = "/wallet/challenges",
    request_body = WalletChallengeRequest,
    responses((status = 200, body = WalletChallengeResponse), (status = 401, body = ApiErrorResponse))
)]
async fn wallet_challenge_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<WalletChallengeRequest>,
) -> Result<Json<WalletChallengeResponse>, ApiError> {
    if matches!(request.provider, WalletProvider::Phantom) {
        return Err(ApiError::from(AuthError::InvalidProvider));
    }
    let chain = SupportedChain::from_eip155_chain_id(request.chain_id)
        .ok_or_else(|| ApiError::from(AuthError::InvalidProvider))?;
    let address = request.address.to_ascii_lowercase();
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(ApiError::from(AuthError::InvalidWalletProof));
    }
    let address_bytes =
        hex::decode(&address[2..]).map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    let address_bytes: [u8; 20] = address_bytes
        .try_into()
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    let checksum_address = siwe::eip55(&address_bytes);
    let challenge_id = random_hex(16).map_err(ApiError::from)?;
    let nonce = siwe::generate_nonce();
    let issued_at = OffsetDateTime::now_utc();
    let expiration_time = issued_at + Duration::seconds(WALLET_CHALLENGE_TTL_SECONDS as i64);
    let issued_at = issued_at
        .format(&Rfc3339)
        .map_err(|_| ApiError::from(state_unavailable()))?;
    let expiration_time = expiration_time
        .format(&Rfc3339)
        .map_err(|_| ApiError::from(state_unavailable()))?;
    let message = format!(
        "{} wants you to sign in with your Ethereum account:\n{}\n\nLink this wallet to your Rabovel brokerage account.\n\nURI: {}/wallet/link\nVersion: 1\nChain ID: {}\nNonce: {}\nIssued At: {}\nExpiration Time: {}",
        state.siwe_domain,
        checksum_address,
        state.public_origin,
        chain.eip155_chain_id().expect("EIP-155 chain"),
        nonce,
        issued_at,
        expiration_time,
    );
    // Validate server configuration before exposing the message to a wallet.
    Message::from_str(&message).map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    let expires_at = unix_now().saturating_add(WALLET_CHALLENGE_TTL_SECONDS);
    state
        .ephemeral
        .put_wallet_challenge(
            &challenge_id,
            WalletChallengeRecord {
                user_id: user.user_id,
                expected_address: address,
                chain,
                provider: request.provider,
                message: message.clone(),
            },
            WALLET_CHALLENGE_TTL_SECONDS,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(WalletChallengeResponse {
        challenge_id,
        message,
        expires_at,
    }))
}

#[utoipa::path(
    post,
    path = "/wallet/solana/challenges",
    request_body = SolanaWalletChallengeRequest,
    responses((status = 200, body = WalletChallengeResponse), (status = 401, body = ApiErrorResponse))
)]
async fn solana_wallet_challenge_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<SolanaWalletChallengeRequest>,
) -> Result<Json<WalletChallengeResponse>, ApiError> {
    let public_key = bs58::decode(&request.address)
        .into_vec()
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    VerifyingKey::from_bytes(&public_key)
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;

    let challenge_id = random_hex(16).map_err(ApiError::from)?;
    let nonce = random_hex(16).map_err(ApiError::from)?;
    let issued_at = OffsetDateTime::now_utc();
    let expiration_time = issued_at + Duration::seconds(WALLET_CHALLENGE_TTL_SECONDS as i64);
    let issued_at = issued_at
        .format(&Rfc3339)
        .map_err(|_| ApiError::from(state_unavailable()))?;
    let expiration_time = expiration_time
        .format(&Rfc3339)
        .map_err(|_| ApiError::from(state_unavailable()))?;
    let message = format!(
        "{} wants you to link this Solana wallet:\n{}\n\nLink this wallet to your Rabovel brokerage account.\n\nURI: {}/wallet/link\nNonce: {}\nIssued At: {}\nExpiration Time: {}",
        state.siwe_domain,
        request.address,
        state.public_origin,
        nonce,
        issued_at,
        expiration_time,
    );
    let expires_at = unix_now().saturating_add(WALLET_CHALLENGE_TTL_SECONDS);
    state
        .ephemeral
        .put_wallet_challenge(
            &challenge_id,
            WalletChallengeRecord {
                user_id: user.user_id,
                expected_address: request.address,
                chain: SupportedChain::Solana,
                provider: WalletProvider::Phantom,
                message: message.clone(),
            },
            WALLET_CHALLENGE_TTL_SECONDS,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(WalletChallengeResponse {
        challenge_id,
        message,
        expires_at,
    }))
}

#[utoipa::path(
    post,
    path = "/wallet/link",
    request_body = WalletLinkRequest,
    responses((status = 200, body = WalletLinkResponse), (status = 401, body = ApiErrorResponse))
)]
async fn wallet_link_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<WalletLinkRequest>,
) -> Result<Json<WalletLinkResponse>, ApiError> {
    if state
        .ephemeral
        .is_wallet_challenge_used(&request.challenge_id)
        .await
        .map_err(ApiError::from)?
    {
        return Err(ApiError::from(AuthError::ReplayDetected));
    }
    let challenge = state
        .ephemeral
        .take_wallet_challenge(&request.challenge_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::InvalidWalletProof))?;
    state
        .ephemeral
        .mark_wallet_challenge_used(&request.challenge_id, WALLET_CHALLENGE_TTL_SECONDS)
        .await
        .map_err(ApiError::from)?;
    if challenge.user_id != user.user_id {
        return Err(ApiError::from(AuthError::WalletNotOwned));
    }
    let signer = if matches!(challenge.chain, SupportedChain::Solana) {
        verify_solana_signature(
            &challenge.expected_address,
            challenge.message.as_bytes(),
            &request.signature,
        )?;
        challenge.expected_address.clone()
    } else {
        state
            .wallet_proof
            .verify(&challenge.message, &request.signature)
            .map_err(ApiError::from)?
    };
    if !signer.eq_ignore_ascii_case(&challenge.expected_address) {
        return Err(ApiError::from(AuthError::InvalidWalletProof));
    }
    let wallet_id = format!("wal_{}", random_hex(16).map_err(ApiError::from)?);
    let wallet = WalletConnection::from_verified_proof(
        wallet_id.clone(),
        user.user_id.clone(),
        challenge.chain,
        signer,
        challenge.provider,
        unix_now(),
    )
    .map_err(ApiError::from)?;
    state
        .repository
        .insert_wallet(wallet.clone())
        .await
        .map_err(ApiError::from)?;
    let mut record = state
        .repository
        .get_user(&user.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    let before_status = state.auth_service.onboarding_status(&record.profile);
    record.profile.record_verified_wallet(wallet_id);
    state
        .repository
        .save_user(record.clone())
        .await
        .map_err(ApiError::from)?;
    let after_status = state.auth_service.onboarding_status(&record.profile);

    let _ = messaging::publish(
        state.event_producer.as_ref(),
        WalletLinked {
            user_id: user.user_id.clone(),
            wallet_id: wallet.wallet_id().to_string(),
            chain: serde_str(&wallet.chain()),
            provider: serde_str(&wallet.provider()),
            verified_at: wallet.verified_at(),
        },
        "order-gateway",
    )
    .await;
    publish_onboarding_change(&state, &user.user_id, before_status, after_status).await;

    Ok(Json(WalletLinkResponse {
        onboarding_status: after_status,
        wallet,
    }))
}

fn verify_solana_signature(address: &str, message: &[u8], signature: &str) -> Result<(), ApiError> {
    let public_key = bs58::decode(address)
        .into_vec()
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    let verifying_key = VerifyingKey::from_bytes(&public_key)
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    let signature = BASE64
        .decode(signature)
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    let signature = Ed25519Signature::from_slice(&signature)
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))?;
    verifying_key
        .verify(message, &signature)
        .map_err(|_| ApiError::from(AuthError::InvalidWalletProof))
}

#[utoipa::path(get, path = "/wallets", responses((status = 200, body = [WalletConnection]), (status = 401, body = ApiErrorResponse)))]
async fn list_wallets_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<WalletConnection>>, ApiError> {
    Ok(Json(
        state
            .repository
            .list_wallets(&user.user_id)
            .await
            .map_err(ApiError::from)?,
    ))
}

async fn investor_solana_wallet(
    state: &GatewayState,
    user_id: &str,
) -> Result<WalletConnection, ApiError> {
    let user = state
        .repository
        .get_user(user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    if !user.principal.has_role(UserRole::Trader) {
        return Err(ApiError::from(AuthError::ForbiddenAction));
    }
    state
        .repository
        .list_wallets(user_id)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .find(|wallet| wallet.chain() == SupportedChain::Solana)
        .ok_or_else(|| ApiError::from(AuthError::MissingWallet))
}

#[utoipa::path(get, path = "/wallet/payment-assets/cngn", responses((status = 200, body = WalletPaymentAssetStatus), (status = 401, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
async fn investor_cngn_account_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<WalletPaymentAssetStatus>, ApiError> {
    let wallet = investor_solana_wallet(&state, &user.user_id).await?;
    state
        .payment_assets
        .cngn_wallet_status(wallet.address())
        .await
        .map(Json)
        .map_err(|message| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(message),
        })
}

#[utoipa::path(post, path = "/wallet/payment-assets/cngn", responses((status = 200, body = WalletPaymentAssetStatus), (status = 401, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
async fn create_investor_cngn_account_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<WalletPaymentAssetStatus>, ApiError> {
    state
        .rate_limiter
        .check(
            &format!("session:{}", user.user_id),
            SESSION_RATE_LIMIT.0,
            SESSION_RATE_LIMIT.1,
        )
        .await
        .map_err(ApiError::from)?;
    let wallet = investor_solana_wallet(&state, &user.user_id).await?;
    state
        .payment_assets
        .ensure_cngn_wallet_account(wallet.address())
        .await
        .map(Json)
        .map_err(|message| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(message),
        })
}

#[utoipa::path(
    post,
    path = "/wallet/authorize",
    request_body = WalletAuthorizationRequest,
    responses((status = 200, body = WalletAuthorizationResponse), (status = 403, body = ApiErrorResponse))
)]
async fn authorize_wallet_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<WalletAuthorizationRequest>,
) -> Result<Json<WalletAuthorizationResponse>, ApiError> {
    state
        .rate_limiter
        .check(
            &format!("session:{}", user.user_id),
            SESSION_RATE_LIMIT.0,
            SESSION_RATE_LIMIT.1,
        )
        .await
        .map_err(ApiError::from)?;
    let record = state
        .repository
        .get_user(&user.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    let wallet = state
        .repository
        .get_wallet(&request.wallet_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::MissingWallet))?;
    state
        .auth_service
        .authorize_wallet_action(
            &record.principal,
            &record.profile,
            &wallet,
            request.action,
            user.mfa_verified_at,
            unix_now(),
        )
        .map_err(ApiError::from)?;
    Ok(Json(WalletAuthorizationResponse { authorized: true }))
}

async fn submit_kyc_case_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<KycCaseRequest>,
) -> Result<Json<KycCaseResponse>, ApiError> {
    state
        .rate_limiter
        .check(
            &format!("session:{}", user.user_id),
            SESSION_RATE_LIMIT.0,
            SESSION_RATE_LIMIT.1,
        )
        .await
        .map_err(ApiError::from)?;
    // Confirms the caller is a real session-bound user before we ever talk
    // to the KYC provider on their behalf.
    state
        .repository
        .get_user(&user.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;

    let submission = KycSubmission {
        user_id: user.user_id.clone(),
        full_name: request.full_name,
        date_of_birth: request.date_of_birth,
        country: request.country,
        document_type: request.document_type,
        document_reference: request.document_reference,
        request_id: random_hex(16).map_err(ApiError::from)?,
    };
    let handle = state
        .kyc_provider
        .submit_case(submission)
        .await
        .map_err(ApiError::from)?;
    state
        .repository
        .create_kyc_case(KycCaseRecord {
            case_id: handle.case_id.clone(),
            user_id: user.user_id.clone(),
            status: KycCaseStatus::Submitted,
            risk_tier: None,
            submitted_at: handle.submitted_at,
            finalized_at: None,
        })
        .await
        .map_err(ApiError::from)?;

    let _ = messaging::publish(
        state.event_producer.as_ref(),
        KycCaseSubmitted {
            case_id: handle.case_id.clone(),
            user_id: user.user_id,
            submitted_at: handle.submitted_at,
        },
        "order-gateway",
    )
    .await;

    Ok(Json(KycCaseResponse {
        case_id: handle.case_id,
        submitted_at: handle.submitted_at,
    }))
}

async fn kyc_webhook_handler(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<StatusCode, ApiError> {
    let signature = headers
        .get("x-kyc-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::from(KycError::InvalidSignature))?;
    let verdict = state
        .kyc_provider
        .verify_webhook(&body, signature)
        .map_err(ApiError::from)?;

    let case = state
        .repository
        .get_kyc_case(&verdict.case_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(KycError::CaseNotFound))?;
    if case.user_id != verdict.user_id {
        return Err(ApiError::from(AuthError::PolicyViolation(
            "kyc verdict user mismatch".to_string(),
        )));
    }

    let mut record = state
        .repository
        .get_user(&verdict.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
    let before_status = state.auth_service.onboarding_status(&record.profile);

    let evidence = verdict
        .clone()
        .into_compliance_evidence(
            record.profile.mfa_enrolled(),
            record.profile.trusted_device(),
        )
        .map_err(ApiError::from)?;
    record.profile.apply_verified_compliance(evidence);
    state
        .repository
        .save_user(record.clone())
        .await
        .map_err(ApiError::from)?;

    let case_status = if verdict.kyc_verified {
        KycCaseStatus::Approved
    } else {
        KycCaseStatus::Rejected
    };
    state
        .repository
        .finalize_kyc_case(
            &verdict.case_id,
            case_status,
            Some(verdict.risk_tier),
            verdict.checked_at,
        )
        .await
        .map_err(ApiError::from)?;

    let after_status = state.auth_service.onboarding_status(&record.profile);

    let _ = messaging::publish(
        state.event_producer.as_ref(),
        KycVerdictReceived {
            case_id: verdict.case_id.clone(),
            user_id: verdict.user_id.clone(),
            kyc_verified: verdict.kyc_verified,
            risk_tier: to_event_risk_tier(verdict.risk_tier),
            checked_at: verdict.checked_at,
        },
        "order-gateway",
    )
    .await;
    publish_onboarding_change(&state, &verdict.user_id, before_status, after_status).await;

    Ok(StatusCode::OK)
}

#[cfg(debug_assertions)]
async fn kyc_sandbox_advance_handler(
    State(state): State<GatewayState>,
    Json(request): Json<SandboxAdvanceRequest>,
) -> Result<StatusCode, ApiError> {
    let sandbox = state
        .sandbox_kyc
        .clone()
        .ok_or_else(|| ApiError::from(AuthError::InvalidProvider))?;
    let (body, signature) = sandbox
        .force_webhook_payload(&request.case_id)
        .map_err(ApiError::from)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-kyc-signature",
        signature
            .parse()
            .map_err(|_| ApiError::from(state_unavailable()))?,
    );
    kyc_webhook_handler(State(state), headers, axum::body::Bytes::from(body)).await
}

async fn portfolio_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<PortfolioSnapshot>, ApiError> {
    state
        .authorize_trading_action(&user, WalletAction::ViewPortfolio)
        .await?;
    let snapshot = state
        .portfolio_trading
        .get_portfolio(&user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(snapshot))
}

#[utoipa::path(get, path = "/investor/catalog", responses((status = 200, body = InvestorCatalogResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
async fn investor_catalog_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<InvestorCatalogResponse>, ApiError> {
    let wallet = investor_solana_wallet(&state, &user.user_id).await?;
    let cngn = state
        .payment_assets
        .cngn_wallet_status(wallet.address())
        .await
        .map_err(|message| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(message),
        })?;
    let candidates = state
        .repository
        .list_minted_assets()
        .await
        .map_err(ApiError::from)?;
    let mut assets = Vec::new();
    for asset in candidates {
        if asset.draft.listing_status != "live" {
            continue;
        }
        let Some(backing) = asset.draft.backing.clone() else {
            continue;
        };
        let Some(operation) = state
            .repository
            .get_asset_setup_operation(&asset.asset_id)
            .await
            .map_err(ApiError::from)?
        else {
            continue;
        };
        if operation.status != "confirmed" {
            continue;
        }
        let inventory = state
            .mint_setup
            .initial_inventory(&asset, &operation, false)
            .await
            .map_err(|error| ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                error: AuthError::PolicyViolation(error.message),
            })?;
        if !inventory.issuance_complete {
            continue;
        }
        let position = state
            .mint_setup
            .investor_position(&operation, wallet.address())
            .await
            .map_err(|error| ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                error: AuthError::PolicyViolation(error.message),
            })?;
        let price_decimals = cngn.decimals.unwrap_or(6);
        let price = state
            .price_feed
            .latest(&asset.draft.ticker, price_decimals)
            .map_err(|message| ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                error: AuthError::PolicyViolation(message),
            })?;
        assets.push(InvestorCatalogAssetResponse {
            asset_id: asset.asset_id,
            name: asset.draft.name,
            ticker: asset.draft.ticker,
            description: asset.draft.metadata.description,
            asset_type: asset.draft.asset_type,
            settlement_currency: asset.draft.settlement_currency,
            network: inventory.network,
            mint_address: inventory.mint_address,
            authorized_units: inventory.authorized_units,
            issuer_inventory: inventory.inventory_balance,
            investor_token_account: position.token_account,
            investor_balance: position.balance,
            investor_account_ready: position.account_ready,
            image_uri: asset.draft.metadata.image_uri,
            disclosure: asset.draft.disclosure,
            backing,
            price_per_unit: price.price_base_units.to_string(),
            price_decimals,
            price_source: price.source,
            price_updated_at: price.as_of,
        });
    }
    Ok(Json(InvestorCatalogResponse {
        wallet_address: wallet.address().into(),
        cngn,
        assets,
    }))
}

#[utoipa::path(post, path = "/investor/quote", request_body = QuoteRequest, responses((status = 200, body = QuoteResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 404, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
async fn investor_quote_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<QuoteRequest>,
) -> Result<Json<QuoteResponse>, ApiError> {
    create_investor_quote(&state, &user, request)
        .await
        .map(Json)
}

async fn create_investor_quote(
    state: &GatewayState,
    user: &AuthenticatedUser,
    request: QuoteRequest,
) -> Result<QuoteResponse, ApiError> {
    state
        .authorize_trading_action(user, WalletAction::TradeSpot)
        .await?;

    let fee_bps: u32 = env::var("RABOVEL_DEMO_FEE_BPS")
        .unwrap_or_else(|_| "100".into()) // Default: 1% = 100 bps
        .parse()
        .map_err(|_| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation("invalid demo fee configuration".into()),
        })?;
    let quote_expiry_seconds: u64 = env::var("RABOVEL_DEMO_QUOTE_EXPIRY_SECONDS")
        .unwrap_or_else(|_| "300".into()) // Default: 5 minutes
        .parse()
        .map_err(|_| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation("invalid demo quote expiry configuration".into()),
        })?;

    // Validate quantity
    let quantity: u64 = request.quantity.parse().map_err(|_| ApiError {
        status: StatusCode::BAD_REQUEST,
        error: AuthError::PolicyViolation("invalid quantity".into()),
    })?;
    if quantity == 0 {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: AuthError::PolicyViolation("quantity must be greater than zero".into()),
        });
    }

    // Get investor's Solana wallet
    let wallet = investor_solana_wallet(state, &user.user_id).await?;

    // Get payment asset status (CNGN for now)
    let cngn = state
        .payment_assets
        .cngn_wallet_status(wallet.address())
        .await
        .map_err(|message| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(message),
        })?;

    // Find the asset
    let candidates = state
        .repository
        .list_minted_assets()
        .await
        .map_err(ApiError::from)?;
    let mut asset_record = None;
    for asset in candidates {
        if asset.asset_id == request.asset_id {
            asset_record = Some(asset);
            break;
        }
    }
    let asset = asset_record.ok_or_else(|| ApiError {
        status: StatusCode::NOT_FOUND,
        error: AuthError::PolicyViolation("asset not found".into()),
    })?;
    let market_price = state
        .price_feed
        .latest(&asset.draft.ticker, cngn.decimals.unwrap_or(6))
        .map_err(|message| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(message),
        })?;
    let buy_price = market_price.price_base_units;

    // Get setup operation
    let operation = state
        .repository
        .get_asset_setup_operation(&asset.asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            error: AuthError::PolicyViolation("asset setup operation not found".into()),
        })?;

    if operation.status != "confirmed" {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: AuthError::PolicyViolation("asset setup is not confirmed".into()),
        });
    }

    // Get inventory
    let inventory = state
        .mint_setup
        .initial_inventory(&asset, &operation, false)
        .await
        .map_err(|error| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(error.message),
        })?;

    if !inventory.issuance_complete {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: AuthError::PolicyViolation("asset inventory issuance is not complete".into()),
        });
    }

    // Get investor position
    let position = state
        .mint_setup
        .investor_position(&operation, wallet.address())
        .await
        .map_err(|error| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(error.message),
        })?;

    // Validate based on side
    let (price_per_unit, payment_mint, payment_token_account, equity_token_account) = match request
        .side
    {
        QuoteSide::Buy => {
            // Check available inventory
            let available_inventory: u64 =
                inventory.inventory_balance.parse().map_err(|_| ApiError {
                    status: StatusCode::SERVICE_UNAVAILABLE,
                    error: AuthError::PolicyViolation("invalid inventory balance".into()),
                })?;
            if quantity > available_inventory {
                return Err(ApiError {
                    status: StatusCode::BAD_REQUEST,
                    error: AuthError::PolicyViolation("insufficient inventory available".into()),
                });
            }
            // Check payment balance
            let payment_balance: u64 = cngn
                .balance_base_units
                .as_deref()
                .unwrap_or("0")
                .parse()
                .map_err(|_| ApiError {
                    status: StatusCode::SERVICE_UNAVAILABLE,
                    error: AuthError::PolicyViolation("invalid payment balance".into()),
                })?;
            let total_cost = quantity.checked_mul(buy_price).ok_or_else(|| ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                error: AuthError::PolicyViolation("quantity overflow".into()),
            })?;
            let fee = total_cost
                .checked_mul(fee_bps as u64)
                .ok_or_else(|| ApiError {
                    status: StatusCode::SERVICE_UNAVAILABLE,
                    error: AuthError::PolicyViolation("fee calculation overflow".into()),
                })?
                / 10000;
            let total_payment = total_cost.checked_add(fee).ok_or_else(|| ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                error: AuthError::PolicyViolation("total payment overflow".into()),
            })?;
            if total_payment > payment_balance {
                return Err(ApiError {
                    status: StatusCode::BAD_REQUEST,
                    error: AuthError::PolicyViolation("insufficient payment balance".into()),
                });
            }
            (
                buy_price.to_string(),
                cngn.mint_address.clone(),
                cngn.token_account.clone().ok_or_else(|| ApiError {
                    status: StatusCode::BAD_REQUEST,
                    error: AuthError::PolicyViolation(
                        "investor payment token account not found".into(),
                    ),
                })?,
                inventory.token_account.clone(),
            )
        }
        QuoteSide::Sell => {
            // Check investor equity balance
            let investor_balance: u64 = position.balance.parse().map_err(|_| ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                error: AuthError::PolicyViolation("invalid investor equity balance".into()),
            })?;
            if quantity > investor_balance {
                return Err(ApiError {
                    status: StatusCode::BAD_REQUEST,
                    error: AuthError::PolicyViolation("insufficient equity balance".into()),
                });
            }
            // For sell, use a demo bid price (e.g., 90% of buy price)
            let bid_price = buy_price * 90 / 100;
            let payment_account = cngn.token_account.clone().ok_or_else(|| ApiError {
                status: StatusCode::BAD_REQUEST,
                error: AuthError::PolicyViolation(
                    "investor payment token account not found".into(),
                ),
            })?;
            if !position.account_ready {
                return Err(ApiError {
                    status: StatusCode::BAD_REQUEST,
                    error: AuthError::PolicyViolation(
                        "investor equity token account is not ready".into(),
                    ),
                });
            }
            (
                bid_price.to_string(),
                cngn.mint_address.clone(),
                payment_account,
                position.token_account.clone(),
            )
        }
    };

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expires_at = created_at + quote_expiry_seconds;

    let quote_id = format!("quote_{}", Uuid::new_v4().simple());

    // Calculate fee and total for response
    let price: u64 = price_per_unit.parse().unwrap_or(0);
    let total_cost = quantity.checked_mul(price).unwrap_or(0);
    let fee = total_cost.checked_mul(fee_bps as u64).unwrap_or(0) / 10000;
    let total_payment = total_cost.checked_add(fee).unwrap_or(0);

    Ok(QuoteResponse {
        quote_id,
        asset_id: request.asset_id,
        side: request.side,
        quantity: request.quantity,
        price_per_unit,
        fee_bps,
        fee_amount: fee.to_string(),
        total_payment: total_payment.to_string(),
        payment_mint,
        payment_token_account,
        equity_token_account,
        expires_at,
        created_at,
        price_source: market_price.source,
        price_updated_at: market_price.as_of,
    })
}

#[utoipa::path(post, path = "/investor/purchase/prepare", request_body = PreparePurchaseRequest, responses((status = 200, body = PreparedPurchaseResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
async fn prepare_investor_purchase_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<PreparePurchaseRequest>,
) -> Result<Json<PreparedPurchaseResponse>, ApiError> {
    let quote = create_investor_quote(
        &state,
        &user,
        QuoteRequest {
            asset_id: request.asset_id.clone(),
            side: QuoteSide::Buy,
            quantity: request.quantity,
        },
    )
    .await?;
    let wallet = investor_solana_wallet(&state, &user.user_id).await?;
    let asset = state
        .repository
        .get_asset_draft(&request.asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            error: AuthError::PolicyViolation("asset not found".into()),
        })?;
    if asset.draft.listing_status != "live" {
        return Err(ApiError {
            status: StatusCode::CONFLICT,
            error: AuthError::PolicyViolation("asset listing is not live".into()),
        });
    }
    let operation = state
        .repository
        .get_asset_setup_operation(&request.asset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError {
            status: StatusCode::CONFLICT,
            error: AuthError::PolicyViolation("asset setup is unavailable".into()),
        })?;
    let payment = state.payment_assets.cngn_status().await;
    if !payment.ready {
        return Err(ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(
                payment
                    .error
                    .unwrap_or_else(|| "broker cNGN account is not ready".into()),
            ),
        });
    }
    let token_program = payment
        .token_program
        .as_deref()
        .and_then(|value| match value {
            "spl-token" => Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            "spl-token-2022" => Some("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
            _ => None,
        })
        .ok_or_else(|| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation("unsupported cNGN token program".into()),
        })?;
    let quantity = quote
        .quantity
        .parse::<u64>()
        .map_err(|_| ApiError::from(AuthError::PolicyViolation("invalid quote quantity".into())))?;
    let total = quote
        .total_payment
        .parse::<u64>()
        .map_err(|_| ApiError::from(AuthError::PolicyViolation("invalid quote total".into())))?;
    let transaction = state
        .mint_setup
        .prepare_purchase(
            &asset,
            &operation,
            PurchaseTerms {
                investor_wallet: wallet.address(),
                investor_payment_account: &quote.payment_token_account,
                payment_mint: &quote.payment_mint,
                payment_token_program: token_program,
                payment_decimals: payment.decimals.unwrap_or(6),
                broker_payment_account: payment.broker_token_account.as_deref().ok_or_else(
                    || ApiError {
                        status: StatusCode::SERVICE_UNAVAILABLE,
                        error: AuthError::PolicyViolation(
                            "broker cNGN account is unavailable".into(),
                        ),
                    },
                )?,
                equity_quantity: quantity,
                payment_amount: total,
            },
        )
        .await
        .map_err(|error| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: AuthError::PolicyViolation(error.message),
        })?;
    Ok(Json(PreparedPurchaseResponse {
        transaction_base64: BASE64.encode(transaction),
        quote,
    }))
}

#[utoipa::path(post, path = "/investor/purchase/submit", request_body = SubmitPurchaseRequest, responses((status = 200, body = PurchaseSettlementResponse), (status = 400, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
async fn submit_investor_purchase_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<SubmitPurchaseRequest>,
) -> Result<Json<PurchaseSettlementResponse>, ApiError> {
    state
        .authorize_trading_action(&user, WalletAction::TradeSpot)
        .await?;
    let wallet = investor_solana_wallet(&state, &user.user_id).await?;
    let transaction = BASE64.decode(request.transaction_base64).map_err(|_| {
        ApiError::from(AuthError::PolicyViolation(
            "signed purchase transaction is not valid base64".into(),
        ))
    })?;
    let signature = state
        .mint_setup
        .submit_purchase(wallet.address(), &transaction)
        .await
        .map_err(|error| ApiError {
            status: if error.reconciliation_required {
                StatusCode::CONFLICT
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            },
            error: AuthError::PolicyViolation(error.message),
        })?;
    Ok(Json(PurchaseSettlementResponse {
        signature,
        status: "confirmed".into(),
    }))
}

async fn place_order_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<NewOrderRequest>,
) -> Result<Json<OrderAck>, ApiError> {
    state
        .rate_limiter
        .check(
            &format!("session:{}", user.user_id),
            SESSION_RATE_LIMIT.0,
            SESSION_RATE_LIMIT.1,
        )
        .await
        .map_err(ApiError::from)?;
    state
        .authorize_trading_action(&user, WalletAction::TradeSpot)
        .await?;
    let ack = state
        .portfolio_trading
        .place_order(&user.user_id, request.clone())
        .await
        .map_err(ApiError::from)?;

    let _ = messaging::publish(
        state.event_producer.as_ref(),
        OrderAccepted {
            order_id: ack.order_id.clone(),
            user_id: user.user_id,
            symbol: request.symbol,
            side: to_event_order_side(request.side),
            quantity: request.quantity,
            accepted_at: ack.accepted_at,
        },
        "order-gateway",
    )
    .await;

    Ok(Json(ack))
}

async fn list_orders_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<OrderStatus>>, ApiError> {
    state
        .authorize_trading_action(&user, WalletAction::ViewPortfolio)
        .await?;
    let orders = state
        .portfolio_trading
        .list_orders(&user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(orders))
}

async fn cancel_order_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Path(order_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .authorize_trading_action(&user, WalletAction::TradeSpot)
        .await?;
    state
        .portfolio_trading
        .cancel_order(&user.user_id, &order_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn tokenization_request_handler(
    State(state): State<GatewayState>,
    user: AuthenticatedUser,
    Json(request): Json<TokenizationRequestBody>,
) -> Result<(StatusCode, Json<TokenizationAccepted>), ApiError> {
    state
        .rate_limiter
        .check(
            &format!("session:{}", user.user_id),
            SESSION_RATE_LIMIT.0,
            SESSION_RATE_LIMIT.1,
        )
        .await
        .map_err(ApiError::from)?;
    state
        .authorize_trading_action(&user, WalletAction::TradeSpot)
        .await?;
    let request_id = format!("tok_{}", random_hex(16).map_err(ApiError::from)?);

    let _ = messaging::publish(
        state.event_producer.as_ref(),
        TokenizationRequested {
            request_id: request_id.clone(),
            user_id: user.user_id,
            asset_ref: request.asset_ref,
            amount: request.amount,
            requested_at: unix_now(),
        },
        "order-gateway",
    )
    .await;

    Ok((
        StatusCode::ACCEPTED,
        Json(TokenizationAccepted { request_id }),
    ))
}

/// Not yet correlated to a real orchestrator's response -- always reports
/// `pending`. See the implementation plan's "future work" note: the BFF
/// would need to consume `TokenizationCompleted`/`Failed` off Kafka and
/// update a status cache to make this accurate.
async fn tokenization_status_handler(
    Path(request_id): Path<String>,
    _user: AuthenticatedUser,
) -> Result<Json<TokenizationStatusResponse>, ApiError> {
    Ok(Json(TokenizationStatusResponse {
        request_id,
        status: "pending".to_string(),
    }))
}

async fn publish_onboarding_change(
    state: &GatewayState,
    user_id: &str,
    before: BrokerageOnboardingStatus,
    after: BrokerageOnboardingStatus,
) {
    if before == after {
        return;
    }
    let _ = messaging::publish(
        state.event_producer.as_ref(),
        OnboardingStatusChanged {
            user_id: user_id.to_string(),
            previous_status: to_event_status(before),
            new_status: to_event_status(after),
            occurred_at: unix_now(),
        },
        "order-gateway",
    )
    .await;
}

fn to_event_status(status: BrokerageOnboardingStatus) -> EventOnboardingStatus {
    match status {
        BrokerageOnboardingStatus::Approved => EventOnboardingStatus::Approved,
        BrokerageOnboardingStatus::RequiresReview => EventOnboardingStatus::RequiresReview,
        BrokerageOnboardingStatus::Blocked => EventOnboardingStatus::Blocked,
    }
}

fn to_event_risk_tier(tier: RiskTier) -> KycRiskTier {
    match tier {
        RiskTier::Standard => KycRiskTier::Standard,
        RiskTier::Enhanced => KycRiskTier::Enhanced,
        // `Unassessed` should never reach here: `VerifiedComplianceEvidence::new`
        // (called by `VerifiedKycVerdict::into_compliance_evidence`) rejects it
        // upstream. Mapped conservatively rather than panicking.
        RiskTier::HighRisk | RiskTier::Unassessed => KycRiskTier::HighRisk,
    }
}

fn to_event_order_side(side: OrderSide) -> EventOrderSide {
    match side {
        OrderSide::Buy => EventOrderSide::Buy,
        OrderSide::Sell => EventOrderSide::Sell,
    }
}

/// Encodes a `serde`-serializable unit-variant enum as its bare string form
/// (the same string domain's `#[serde(rename_all = "snake_case")]` already
/// produces), for embedding in an event payload's `String` fields.
fn serde_str<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn random_hex(bytes: usize) -> Result<String, AuthError> {
    let mut value = vec![0_u8; bytes];
    getrandom::getrandom(&mut value).map_err(|_| state_unavailable())?;
    Ok(hex::encode(value))
}

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get("authorization")?.to_str().ok()?;
    value
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}

fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("authentication state is unavailable".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    struct FakeGoogle;

    #[async_trait]
    impl GoogleIdentityProvider for FakeGoogle {
        fn begin(&self) -> Result<OidcStart, AuthError> {
            Ok(OidcStart {
                authorization_url: "https://accounts.example/authorize?state=test-state"
                    .to_string(),
                state: "test-state".to_string(),
                pending: OidcPending {
                    nonce: "test-nonce".to_string(),
                    pkce_verifier: "test-pkce-verifier".to_string(),
                },
            })
        }

        async fn exchange(
            &self,
            code: &str,
            pending: &OidcPending,
        ) -> Result<VerifiedGoogleIdentity, AuthError> {
            if code != "valid-code"
                || pending.nonce != "test-nonce"
                || pending.pkce_verifier != "test-pkce-verifier"
            {
                return Err(AuthError::InvalidIdentityProof);
            }
            VerifiedGoogleIdentity::from_verified_claims(
                "https://accounts.google.com",
                "subject-1",
                "user@example.com",
                true,
            )
        }
    }

    struct FakeWalletProof;

    impl WalletProofVerifier for FakeWalletProof {
        fn verify(&self, message: &str, signature: &str) -> Result<String, AuthError> {
            if signature != "valid-test-signature" {
                return Err(AuthError::InvalidWalletProof);
            }
            let message = Message::from_str(message).map_err(|_| AuthError::InvalidWalletProof)?;
            Ok(format!("0x{}", hex::encode(message.address)))
        }
    }

    #[derive(Default)]
    struct FakeMintSetup {
        issued: std::sync::atomic::AtomicBool,
    }

    struct FakePaymentAssets;

    #[async_trait]
    impl PaymentAssetRegistry for FakePaymentAssets {
        async fn cngn_status(&self) -> PaymentAssetStatus {
            PaymentAssetStatus {
                code: "CNGN".into(),
                name: "cNGN".into(),
                network: "testnet".into(),
                mint_address: Some("test-mint".into()),
                token_program: Some("spl-token-2022".into()),
                decimals: Some(6),
                supply_base_units: Some("1000000000".into()),
                broker_owner_address: Some("broker".into()),
                broker_token_account: Some("broker-account".into()),
                broker_balance_base_units: Some("100000000".into()),
                fee_payer_address: Some("admin".into()),
                mint_verified: true,
                broker_account_verified: true,
                ready: true,
                error: None,
            }
        }

        async fn ensure_cngn_broker_account(&self) -> Result<PaymentAssetStatus, String> {
            Ok(self.cngn_status().await)
        }

        async fn cngn_wallet_status(
            &self,
            owner: &str,
        ) -> Result<WalletPaymentAssetStatus, String> {
            Ok(WalletPaymentAssetStatus {
                code: "CNGN".into(),
                name: "cNGN".into(),
                network: "devnet".into(),
                mint_address: "test-mint".into(),
                token_program: Some("spl-token-2022".into()),
                decimals: Some(6),
                wallet_address: owner.into(),
                token_account: Some("investor-cngn-account".into()),
                balance_base_units: Some("25000000".into()),
                account_verified: true,
                ready: true,
                error: None,
            })
        }

        async fn ensure_cngn_wallet_account(
            &self,
            owner: &str,
        ) -> Result<WalletPaymentAssetStatus, String> {
            self.cngn_wallet_status(owner).await
        }
    }

    #[async_trait]
    impl MintSetupExecutor for FakeMintSetup {
        fn is_configured(&self) -> bool {
            true
        }
        fn network(&self) -> &str {
            "devnet"
        }
        fn mint_address(&self, operation_id: &str) -> String {
            format!("mint-{operation_id}")
        }
        async fn execute(
            &self,
            _: &AssetDraftRecord,
            _: &AssetSetupOperation,
        ) -> Result<SetupExecution, SetupExecutionError> {
            Ok(SetupExecution {
                mint_signature: Some("mint-signature".into()),
                shared_lists_signature: Some("lists-signature".into()),
                acl_signatures: vec!["acl-signature".into()],
                mint_config_address: "mint-config".into(),
                allow_list_address: "allow-list".into(),
                block_list_address: "block-list".into(),
                thaw_extra_metas_address: "thaw-extra-metas".into(),
            })
        }
        async fn initial_inventory(
            &self,
            asset: &AssetDraftRecord,
            operation: &AssetSetupOperation,
            issue: bool,
        ) -> Result<InitialInventory, SetupExecutionError> {
            if issue {
                self.issued.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            let issued = self.issued.load(std::sync::atomic::Ordering::SeqCst);
            let amount = if issued {
                asset.draft.authorized_units.to_string()
            } else {
                "0".into()
            };
            Ok(InitialInventory {
                network: operation.network.clone(),
                mint_address: operation.mint_address.clone(),
                settlement_wallet: "issuer-settlement-wallet".into(),
                token_account: "issuer-inventory-account".into(),
                authorized_units: asset.draft.authorized_units.to_string(),
                supply: amount.clone(),
                inventory_balance: amount,
                wallet_allowlisted: issued,
                token_account_ready: issued,
                issuance_complete: issued,
                signatures: issue
                    .then(|| "inventory-signature".into())
                    .into_iter()
                    .collect(),
            })
        }
        async fn investor_position(
            &self,
            _: &AssetSetupOperation,
            _: &str,
        ) -> Result<InvestorAssetPosition, SetupExecutionError> {
            Ok(InvestorAssetPosition {
                token_account: "investor-equity-account".into(),
                balance: "0".into(),
                account_ready: true,
            })
        }
        async fn prepare_purchase(
            &self,
            _: &AssetDraftRecord,
            _: &AssetSetupOperation,
            _: PurchaseTerms<'_>,
        ) -> Result<Vec<u8>, SetupExecutionError> {
            Err(SetupExecutionError {
                message: "purchase preparation is not available in the unit fake".into(),
                reconciliation_required: false,
            })
        }
        async fn submit_purchase(&self, _: &str, _: &[u8]) -> Result<String, SetupExecutionError> {
            Err(SetupExecutionError {
                message: "purchase submission is not available in the unit fake".into(),
                reconciliation_required: false,
            })
        }
    }

    fn test_app() -> Router {
        router_with_state(GatewayState::with_providers(
            Arc::new(FakeGoogle),
            Arc::new(FakeWalletProof),
            "localhost:3000".to_string(),
            "http://localhost:3000".to_string(),
        ))
    }

    fn test_app_with_role(role: UserRole) -> Router {
        router_with_state(
            GatewayStateBuilder::default()
                .google(Arc::new(FakeGoogle))
                .wallet_proof(Arc::new(FakeWalletProof))
                .trusted_role_assignments(vec![TrustedRoleAssignment {
                    issuer: "https://accounts.google.com".to_string(),
                    subject: "subject-1".to_string(),
                    role,
                }])
                .metadata_storage(Arc::new(InMemoryMetadataStorage))
                .mint_setup(Arc::new(FakeMintSetup::default()))
                .payment_assets(Arc::new(FakePaymentAssets))
                .security_policy(test_security_policy())
                .build(),
        )
    }

    fn test_security_policy() -> BrokerageSecurityPolicy {
        BrokerageSecurityPolicy::for_testing()
    }

    async fn body_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn login(app: Router) -> (Router, String) {
        let start = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/google/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(start.status(), StatusCode::OK);
        let callback = serde_json::json!({"code": "valid-code", "state": "test-state"});
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/google/callback")
                    .header("content-type", "application/json")
                    .body(Body::from(callback.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        (app, body["access_token"].as_str().unwrap().to_string())
    }

    async fn register_password_user(app: Router, email: &str) -> (Router, String) {
        let payload = serde_json::json!({
            "display_name": "Demo Investor",
            "email": email,
            "password": "correct-horse-battery"
        });
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        (app, body["access_token"].as_str().unwrap().to_string())
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let response = router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn fail_closed_router_preserves_configured_cors_origin() {
        let response = fail_closed_router("http://localhost:3001")
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/auth/google/start")
                    .header("origin", "http://localhost:3001")
                    .header("access-control-request-method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("access-control-allow-origin")
                .unwrap(),
            "http://localhost:3001"
        );
    }

    #[tokio::test]
    async fn old_self_asserted_google_payload_is_rejected() {
        let payload = serde_json::json!({
            "user_id": "attacker",
            "email_verified": true,
            "google_subject": "forged-subject"
        });
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/google/callback")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn callback_rejects_client_supplied_roles() {
        let payload = serde_json::json!({
            "code": "valid-code",
            "state": "test-state",
            "role": "admin"
        });
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/google/callback")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn password_registration_and_login_use_server_assigned_roles() {
        let app = router_with_state(
            GatewayStateBuilder::default()
                .trusted_email_roles(vec![("admin@example.com".to_string(), UserRole::Admin)])
                .build(),
        );
        let (app, investor_token) = register_password_user(app, "Investor@Example.com").await;
        let investor_me = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header("authorization", format!("Bearer {investor_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            body_json(investor_me).await["roles"],
            serde_json::json!(["trader"])
        );

        let (app, admin_token) = register_password_user(app, "admin@example.com").await;
        let admin_me = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header("authorization", format!("Bearer {admin_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            body_json(admin_me).await["roles"],
            serde_json::json!(["admin"])
        );

        let login = serde_json::json!({"email": "investor@example.com", "password": "correct-horse-battery"});
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(login.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn password_auth_rejects_wrong_password_and_role_fields() {
        let (app, _) = register_password_user(test_app(), "user@example.com").await;
        let wrong = serde_json::json!({"email": "user@example.com", "password": "wrong-password"});
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(wrong.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let forged = serde_json::json!({"display_name": "Attacker", "email": "attacker@example.com", "password": "password123", "role": "admin"});
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(forged.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn auth_me_requires_a_live_session_and_logout_revokes_it() {
        let missing = test_app()
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

        let (app, token) = login(test_app()).await;
        let me = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(me.status(), StatusCode::OK);
        assert_eq!(body_json(me).await["roles"], serde_json::json!(["trader"]));

        let logout = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(logout.status(), StatusCode::NO_CONTENT);
        let rejected = app
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_me_rejects_an_expired_session() {
        let ephemeral = Arc::new(InMemoryEphemeralStore::default());
        let token = "expired-token";
        ephemeral
            .create_session(
                &hash_token(token),
                SessionRecord {
                    user_id: "usr_expired".to_string(),
                    mfa_verified_at: None,
                },
                0,
            )
            .await
            .unwrap();
        let app = router_with_state(GatewayStateBuilder::default().ephemeral(ephemeral).build());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn trusted_roles_are_assigned_and_issuer_endpoint_is_authoritative() {
        let (issuer_app, issuer_token) = login(test_app_with_role(UserRole::Issuer)).await;
        let overview = issuer_app
            .oneshot(
                Request::builder()
                    .uri("/issuer/overview")
                    .header("authorization", format!("Bearer {issuer_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(overview.status(), StatusCode::OK);

        let (investor_app, investor_token) = login(test_app()).await;
        let forbidden = investor_app
            .oneshot(
                Request::builder()
                    .uri("/issuer/overview")
                    .header("authorization", format!("Bearer {investor_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

        let (admin_app, admin_token) = login(test_app_with_role(UserRole::Admin)).await;
        let me = admin_app
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header("authorization", format!("Bearer {admin_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(body_json(me).await["roles"], serde_json::json!(["admin"]));
    }

    #[tokio::test]
    async fn approved_issuer_creates_and_reads_valid_asset_draft() {
        let (app, token) = login(test_app_with_role(UserRole::Issuer)).await;
        let payment_asset = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/payment-assets/cngn")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(payment_asset.status(), StatusCode::OK);
        let payment_asset = body_json(payment_asset).await;
        assert_eq!(payment_asset["network"], "testnet");
        assert_eq!(payment_asset["decimals"], 6);
        assert_eq!(payment_asset["ready"], true);
        let organization = serde_json::json!({
            "legal_name": "Demo Issuer Ltd", "organization_type": "company", "registration_number": "RC-123",
            "jurisdiction": "Nigeria", "registered_address": "1 Demo Street", "representative_name": "Ada Demo",
            "representative_title": "Director", "document_reference": "demo-registration.pdf",
            "beneficial_owners_confirmed": true, "information_certified": true
        });
        let onboarded = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/issuer/onboarding")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(organization.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(onboarded.status(), StatusCode::OK);

        let broker_account = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/payment-assets/cngn")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(broker_account.status(), StatusCode::OK);
        let broker_account = body_json(broker_account).await;
        assert_eq!(broker_account["broker_account_verified"], true);
        assert_eq!(broker_account["fee_payer_address"], "admin");

        let draft = serde_json::json!({
            "instrument_code": "RABO-NG-TELCO", "name": "Rabovel Nigeria Telco", "ticker": "RABO-NG-TELCO",
            "market": "NGX", "share_class": "Ordinary", "asset_type": "equity", "decimals": 0,
            "authorized_units": "1000000", "settlement_currency": "CNGN",
            "representation": "1 token = 1 simulated beneficial entitlement to 1 share",
            "rights_description": "Simulated ordinary-share entitlement for the Rabovel prototype.",
            "disclosure": domain::assets::DEMO_DISCLOSURE,
            "metadata": { "name": "Rabovel Nigeria Telco", "symbol": "RABO-NG-TELCO", "description": "Demo equity", "image_uri": null, "external_url": null, "metadata_uri": null, "additional_metadata": [] }
        });
        let created = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/issuer/assets")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(draft.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::OK);

        let created = body_json(created).await;
        assert_eq!(created["status"], "draft");
        assert_eq!(created["issued_units"], "0");
        assert!(created["mint_address"].is_null());
        let asset_id = created["asset_id"].as_str().unwrap();

        let mut revised = draft.clone();
        revised["name"] = serde_json::json!("Rabovel Nigeria Telco Revised");
        revised["metadata"]["name"] = serde_json::json!("Rabovel Nigeria Telco Revised");
        let updated = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/issuer/assets/{asset_id}"))
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(revised.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(updated.status(), StatusCode::OK);
        assert_eq!(
            body_json(updated).await["draft"]["name"],
            "Rabovel Nigeria Telco Revised"
        );

        let listed = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/issuer/assets")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(listed.status(), StatusCode::OK);
        assert_eq!(body_json(listed).await.as_array().unwrap().len(), 1);

        let duplicate = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/issuer/assets")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(draft.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(duplicate.status(), StatusCode::BAD_REQUEST);

        let submitted = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/issuer/assets/{asset_id}/submit"))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(submitted.status(), StatusCode::OK);
        let submitted = body_json(submitted).await;
        assert_eq!(submitted["status"], "approved_for_setup");
        assert_eq!(
            submitted["draft"]["metadata"]["metadata_uri"],
            format!("https://metadata.example/assets/{asset_id}/metadata.json")
        );

        let setup = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/issuer/assets/{asset_id}/setup"))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(setup.status(), StatusCode::OK);
        let setup = body_json(setup).await;
        assert_eq!(setup["status"], "confirmed");
        assert_eq!(setup["stages"].as_array().unwrap().len(), 4);
        let operation_id = setup["operation_id"].as_str().unwrap();
        assert_eq!(setup["mint_address"], format!("mint-{operation_id}"));

        let replay = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/issuer/assets/{asset_id}/setup"))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(body_json(replay).await["operation_id"], operation_id);

        let minted = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/issuer/assets/{asset_id}"))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let minted = body_json(minted).await;
        assert_eq!(minted["status"], "minted");
        assert_eq!(minted["mint_address"], format!("mint-{operation_id}"));

        let inventory = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/issuer/assets/{asset_id}/inventory"))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(inventory.status(), StatusCode::OK);
        let inventory = body_json(inventory).await;
        assert_eq!(inventory["issuance_complete"], true);
        assert_eq!(inventory["wallet_allowlisted"], true);
        assert_eq!(inventory["inventory_balance"], "1000000");

        let backing = serde_json::json!({
            "summary": "Each token represents one simulated ordinary share held by the issuer.",
            "document_name": "share-register.pdf",
            "content_type": "application/pdf",
            "size_bytes": 2048
        });
        let backed = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/issuer/assets/{asset_id}/backing"))
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(backing.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(backed.status(), StatusCode::OK);
        let backed = body_json(backed).await;
        assert_eq!(
            backed["draft"]["backing"]["verification_status"],
            "verified"
        );
        assert_eq!(backed["draft"]["listing_status"], "not_listed");

        let published = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/issuer/assets/{asset_id}/listing"))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(published.status(), StatusCode::OK);
        assert_eq!(
            body_json(published).await["draft"]["listing_status"],
            "live"
        );

        let edit_after_approval = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/issuer/assets/{asset_id}"))
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(revised.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(edit_after_approval.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn oidc_state_is_single_use() {
        let (app, _) = login(test_app()).await;
        let callback = serde_json::json!({"code": "valid-code", "state": "test-state"});
        let replay = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/google/callback")
                    .header("content-type", "application/json")
                    .body(Body::from(callback.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(replay.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn wallet_authorization_requires_a_server_session() {
        let request = serde_json::json!({
            "wallet_id": "wal_attacker",
            "action": "sign:withdrawal"
        });
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/authorize")
                    .header("content-type", "application/json")
                    .body(Body::from(request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn old_forged_principal_and_verified_wallet_are_rejected() {
        let (app, token) = login(test_app()).await;
        let forged = serde_json::json!({
            "principal": {
                "user_id": "attacker",
                "roles": ["admin"],
                "permissions": ["sign:withdrawal"]
            },
            "wallet": {
                "chain": "ethereum",
                "address": "0x1111111111111111111111111111111111111111",
                "provider": "meta_mask",
                "verified": true
            },
            "action": "sign:withdrawal"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/authorize")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(forged.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn wallet_challenge_is_session_bound_and_single_use() {
        let (app, token) = login(test_app()).await;
        let request = serde_json::json!({
            "address": "0x6da01670d8fc844e736095918bbe11fe8d564163",
            "chain_id": 1,
            "provider": "meta_mask"
        });
        let challenge = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/challenges")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(challenge.status(), StatusCode::OK);
        let challenge = body_json(challenge).await;
        let link = serde_json::json!({
            "challenge_id": challenge["challenge_id"],
            "signature": "valid-test-signature"
        });
        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/link")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(link.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let replay = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/link")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(link.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(replay.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn phantom_wallet_is_verified_persisted_and_listed() {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[7_u8; 32]);
        let address = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
        let (app, token) = login(test_app_with_role(UserRole::Trader)).await;
        let challenge = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/solana/challenges")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "address": address }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(challenge.status(), StatusCode::OK);
        let challenge = body_json(challenge).await;
        let message = challenge["message"].as_str().unwrap();
        let signature = ed25519_dalek::Signer::sign(&signing_key, message.as_bytes());
        let linked = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/link")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "challenge_id": challenge["challenge_id"],
                            "signature": BASE64.encode(signature.to_bytes())
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(linked.status(), StatusCode::OK);
        let linked = body_json(linked).await;
        assert_eq!(linked["wallet"]["chain"], "solana");
        assert_eq!(linked["wallet"]["provider"], "phantom");
        assert_eq!(linked["wallet"]["address"], address);

        let wallets = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/wallets")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(wallets.status(), StatusCode::OK);
        let wallets = body_json(wallets).await;
        assert_eq!(wallets.as_array().unwrap().len(), 1);
        assert_eq!(wallets[0]["address"], address);

        let payment_account = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/wallet/payment-assets/cngn")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(payment_account.status(), StatusCode::OK);
        let payment_account = body_json(payment_account).await;
        assert_eq!(payment_account["wallet_address"], address);
        assert_eq!(payment_account["token_account"], "investor-cngn-account");
        assert_eq!(payment_account["balance_base_units"], "25000000");

        let created = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/payment-assets/cngn")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::OK);

        let catalog = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/investor/catalog")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(catalog.status(), StatusCode::OK);
        let catalog = body_json(catalog).await;
        assert_eq!(catalog["wallet_address"], address);
        assert_eq!(catalog["cngn"]["balance_base_units"], "25000000");
        assert_eq!(catalog["assets"].as_array().unwrap().len(), 0);

        // Test quote endpoint - buy
        let quote_request = serde_json::json!({
            "asset_id": "asset-1",
            "side": "buy",
            "quantity": "100"
        });
        let quote = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/investor/quote")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(quote_request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        // Should return 404 because the fake asset doesn't exist in the fake mint setup
        // (FakeMintSetup returns a mint address that doesn't match "asset-1")
        let quote_body = body_json(quote).await;
        eprintln!(
            "Quote response status: {:?}, body: {:?}",
            StatusCode::NOT_FOUND,
            quote_body
        );
        assert_eq!(quote_body["code"], "policy_violation");

        // Test quote endpoint - sell
        let quote_request = serde_json::json!({
            "asset_id": "asset-1",
            "side": "sell",
            "quantity": "100"
        });
        let quote = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/investor/quote")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(quote_request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(quote.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn production_siwe_verifier_recovers_expected_address() {
        let message = r#"localhost:4361 wants you to sign in with your Ethereum account:
0x6Da01670d8fc844e736095918bbE11fE8D564163

SIWE Notepad Example

URI: http://localhost:4361
Version: 1
Chain ID: 1
Nonce: kEWepMt9knR6lWJ6A
Issued At: 2021-12-07T18:28:18.807Z"#;
        let signature = "6228b3ecd7bf2df018183aeab6b6f1db1e9f4e3cbe24560404112e25363540eb679934908143224d746bbb5e1aa65ab435684081f4dbb74a0fec57f98f40f5051c";
        let recovered = SiweWalletProofVerifier.verify(message, signature).unwrap();
        assert_eq!(recovered, "0x6da01670d8fc844e736095918bbe11fe8d564163");
    }

    #[tokio::test]
    async fn kyc_case_lifecycle_flips_onboarding_from_requires_review_toward_approved() {
        let (app, token) = login(test_app()).await;
        // Link a wallet and accept terms first so KYC approval is the last
        // gate standing between this profile and `Approved`.
        let challenge = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/challenges")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "address": "0x6da01670d8fc844e736095918bbe11fe8d564163",
                            "chain_id": 1,
                            "provider": "meta_mask"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let challenge = body_json(challenge).await;
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wallet/link")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "challenge_id": challenge["challenge_id"],
                            "signature": "valid-test-signature"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/onboarding/terms")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"version": "2026-01"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let submit = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/kyc/cases")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "full_name": "Ada Lovelace",
                            "date_of_birth": "1990-01-01",
                            "country": "NG",
                            "document_type": "passport",
                            "document_reference": "vendor-ref-1"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(submit.status(), StatusCode::OK);
        let submit = body_json(submit).await;
        let case_id = submit["case_id"].as_str().unwrap().to_string();

        let advance = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/kyc/_sandbox/advance")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"case_id": case_id}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(advance.status(), StatusCode::OK);

        let status = app
            .oneshot(
                Request::builder()
                    .uri("/onboarding/status")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(status.status(), StatusCode::OK);
        let status = body_json(status).await;
        // MFA is still not enrolled in this test's profile, so full
        // `Approved` isn't reached, but KYC itself must now read verified.
        assert_eq!(status["profile"]["kyc_verified"], true);
    }

    #[tokio::test]
    async fn ip_rate_limit_blocks_after_the_configured_window() {
        let app = test_app();
        for _ in 0..IP_RATE_LIMIT.0 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/health")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let throttled = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(throttled.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
