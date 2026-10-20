use serde::Serialize;
use utoipa::ToSchema;

/// A catalog instrument in the frontend `Asset` shape. Assets and
/// instruments are the same catalog.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssetResponse {
    pub asset_id: String,
    pub market_id: String,
    pub origin: crate::InstrumentOrigin,
    pub issuer_id: String,
    pub issuer_name: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub asset_class: String,
    pub settlement_currency: String,
    pub nav: String,
    pub market_price: String,
    pub total_issued: Option<String>,
    pub lifecycle_status: String,
    pub blockchain: String,
    pub token_standard: String,
    pub contract_address: Option<String>,
}
