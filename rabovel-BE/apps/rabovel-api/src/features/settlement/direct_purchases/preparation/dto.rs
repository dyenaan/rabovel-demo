use crate::features::settlement::direct_purchases::quotes::dto::QuoteResponse;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

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
