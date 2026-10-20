use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

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
