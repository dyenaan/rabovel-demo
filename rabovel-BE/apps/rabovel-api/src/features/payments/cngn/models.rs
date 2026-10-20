use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaymentAssetStatus {
    pub code: String,
    pub name: String,
    pub network: String,
    pub mint_address: Option<String>,
    pub token_program: Option<String>,
    pub decimals: Option<u8>,
    pub supply_base_units: Option<String>,
    pub broker_owner_address: Option<String>,
    pub broker_token_account: Option<String>,
    pub broker_balance_base_units: Option<String>,
    pub fee_payer_address: Option<String>,
    pub mint_verified: bool,
    pub broker_account_verified: bool,
    pub ready: bool,
    #[serde(serialize_with = "crate::http::observability::serialize_provider_error")]
    pub error: Option<String>,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WalletPaymentAssetStatus {
    pub code: String,
    pub name: String,
    pub network: String,
    pub mint_address: String,
    pub token_program: Option<String>,
    pub decimals: Option<u8>,
    pub wallet_address: String,
    pub token_account: Option<String>,
    pub balance_base_units: Option<String>,
    pub account_verified: bool,
    pub ready: bool,
    #[serde(serialize_with = "crate::http::observability::serialize_provider_error")]
    pub error: Option<String>,
}
