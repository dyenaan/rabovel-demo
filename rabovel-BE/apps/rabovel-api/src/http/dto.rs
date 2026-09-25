use domain::assets::{AssetMetadata, NewAssetDraft};
use domain::auth::{
    BrokerageOnboardingStatus, IdentityProfile, UserRole, WalletAction, WalletConnection,
    WalletProvider,
};
use kyc::DocumentType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GoogleAuthorizationResponse {
    pub authorization_url: String,
    pub state: String,
    pub expires_in: u64,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GoogleCallbackRequest {
    pub code: String,
    pub state: String,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PasswordRegisterRequest {
    pub display_name: String,
    pub email: String,
    pub password: String,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PasswordLoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthSessionResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub profile: IdentityProfile,
    pub status: BrokerageOnboardingStatus,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentUserResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<UserRole>,
    pub onboarding_status: BrokerageOnboardingStatus,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct IssuerOverviewResponse {
    pub user_id: String,
    pub email: String,
    pub account_status: String,
    pub organization: Option<IssuerOrganizationResponse>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct IssuerOrganizationResponse {
    pub legal_name: String,
    pub organization_type: String,
    pub registration_number: String,
    pub jurisdiction: String,
    pub registered_address: String,
    pub representative_name: String,
    pub representative_title: String,
    pub document_reference: String,
    pub approved_at: u64,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct IssuerOnboardingRequest {
    pub legal_name: String,
    pub organization_type: String,
    pub registration_number: String,
    pub jurisdiction: String,
    pub registered_address: String,
    pub representative_name: String,
    pub representative_title: String,
    pub document_reference: String,
    pub beneficial_owners_confirmed: bool,
    pub information_certified: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateAssetDraftRequest {
    pub instrument_code: String,
    pub name: String,
    pub ticker: String,
    pub market: String,
    pub share_class: String,
    pub asset_type: String,
    pub decimals: u8,
    /// Integer base units, encoded as a string to preserve exact values in browsers.
    pub authorized_units: String,
    pub settlement_currency: String,
    pub representation: String,
    pub rights_description: String,
    pub disclosure: String,
    pub metadata: AssetMetadata,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AssetDraftResponse {
    pub asset_id: String,
    pub issuer_user_id: String,
    pub status: String,
    pub issued_units: String,
    pub mint_address: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub draft: NewAssetDraft,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct SetupStageResponse {
    pub stage: String,
    pub status: String,
    pub signatures: Vec<String>,
    pub verified_at: Option<u64>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct AssetSetupOperationResponse {
    pub operation_id: String,
    pub asset_id: String,
    pub network: String,
    pub mint_address: String,
    pub status: String,
    pub stages: Vec<SetupStageResponse>,
    pub mint_config_address: Option<String>,
    pub allow_list_address: Option<String>,
    pub block_list_address: Option<String>,
    pub thaw_extra_metas_address: Option<String>,
    pub error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct InitialInventoryResponse {
    pub network: String,
    pub mint_address: String,
    pub settlement_wallet: String,
    pub token_account: String,
    pub authorized_units: String,
    pub supply: String,
    pub inventory_balance: String,
    pub wallet_allowlisted: bool,
    pub token_account_ready: bool,
    pub issuance_complete: bool,
    pub signatures: Vec<String>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct InvestorCatalogAssetResponse {
    pub asset_id: String,
    pub name: String,
    pub ticker: String,
    pub description: String,
    pub asset_type: String,
    pub settlement_currency: String,
    pub network: String,
    pub mint_address: String,
    pub authorized_units: String,
    pub issuer_inventory: String,
    pub investor_token_account: String,
    pub investor_balance: String,
    pub investor_account_ready: bool,
    pub image_uri: Option<String>,
    pub disclosure: String,
    pub backing: domain::assets::BackingEvidence,
    pub price_per_unit: String,
    pub price_decimals: u8,
    pub price_source: String,
    pub price_updated_at: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SubmitBackingRequest {
    pub summary: String,
    pub document_name: String,
    pub content_type: String,
    pub size_bytes: u64,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct InvestorCatalogResponse {
    pub wallet_address: String,
    pub cngn: crate::payment_assets::WalletPaymentAssetStatus,
    pub assets: Vec<InvestorCatalogAssetResponse>,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageUploadRequest {
    pub content_type: String,
    pub size: u64,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct ImageUploadTicket {
    pub signed_url: String,
    pub object_path: String,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfirmImageUploadRequest {
    pub object_path: String,
    pub content_type: String,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct OnboardingStatusResponse {
    pub profile: IdentityProfile,
    pub status: BrokerageOnboardingStatus,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TermsAcceptanceRequest {
    pub version: String,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WalletChallengeRequest {
    pub address: String,
    pub chain_id: u64,
    pub provider: WalletProvider,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SolanaWalletChallengeRequest {
    pub address: String,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct WalletChallengeResponse {
    pub challenge_id: String,
    pub message: String,
    pub expires_at: u64,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WalletLinkRequest {
    pub challenge_id: String,
    pub signature: String,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct WalletLinkResponse {
    pub wallet: WalletConnection,
    pub onboarding_status: BrokerageOnboardingStatus,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WalletAuthorizationRequest {
    pub wallet_id: String,
    pub action: WalletAction,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct WalletAuthorizationResponse {
    pub authorized: bool,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KycCaseRequest {
    pub full_name: String,
    pub date_of_birth: String,
    pub country: String,
    pub document_type: DocumentType,
    pub document_reference: String,
}
#[derive(Debug, Serialize)]
pub struct KycCaseResponse {
    pub case_id: String,
    pub submitted_at: u64,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg(debug_assertions)]
pub(crate) struct SandboxAdvanceRequest {
    pub(crate) case_id: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenizationRequestBody {
    pub asset_ref: String,
    pub amount: String,
}
#[derive(Debug, Serialize)]
pub struct TokenizationAccepted {
    pub request_id: String,
}
#[derive(Debug, Serialize)]
pub struct TokenizationStatusResponse {
    pub request_id: String,
    pub status: String,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct QuoteRequest {
    pub asset_id: String,
    pub side: QuoteSide,
    pub quantity: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuoteSide {
    Buy,
    Sell,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct QuoteResponse {
    pub quote_id: String,
    pub asset_id: String,
    pub side: QuoteSide,
    pub quantity: String,
    pub price_per_unit: String,
    pub fee_bps: u32,
    pub fee_amount: String,
    pub total_payment: String,
    pub payment_mint: String,
    pub payment_token_account: String,
    pub equity_token_account: String,
    pub expires_at: u64,
    pub created_at: u64,
    pub price_source: String,
    pub price_updated_at: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparePurchaseRequest {
    pub asset_id: String,
    pub quantity: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PreparedPurchaseResponse {
    pub transaction_base64: String,
    pub quote: QuoteResponse,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SubmitPurchaseRequest {
    pub transaction_base64: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PurchaseSettlementResponse {
    pub signature: String,
    pub status: String,
}
