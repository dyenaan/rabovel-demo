use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

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
