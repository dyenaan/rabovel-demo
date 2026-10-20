use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSummaryResponse {
    pub total_value: String,
    pub total_cost_basis: String,
    pub total_unrealized_pnl: String,
    pub total_unrealized_pnl_percent: String,
    pub cash_balance: String,
    pub invested_value: String,
    pub ytd_income: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HoldingResponse {
    pub asset_id: String,
    pub asset_name: String,
    pub symbol: String,
    pub quantity: String,
    pub average_cost: String,
    pub market_value: String,
    pub cost_basis: String,
    pub unrealized_pnl: String,
    pub unrealized_pnl_percent: String,
    pub allocation_percent: String,
}
