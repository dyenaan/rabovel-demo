use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PerformancePointResponse {
    pub date: String,
    pub portfolio_value: String,
}
