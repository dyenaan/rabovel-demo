use serde::Serialize;
use utoipa::ToSchema;

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
    pub backing: issuer_assets::assets::drafts::models::BackingEvidence,
    pub price_per_unit: String,
    pub price_decimals: u8,
    pub price_source: String,
    pub price_updated_at: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvestorCatalogResponse {
    pub wallet_address: String,
    pub cngn: crate::features::payments::cngn::models::WalletPaymentAssetStatus,
    pub assets: Vec<InvestorCatalogAssetResponse>,
}
