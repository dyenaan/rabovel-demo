/// A snapshot of a user's portfolio at a point in time.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortfolioSnapshot {
    pub user_id: String,
    pub balances: Vec<AssetBalance>,
}
/// A single asset's balance breakdown.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetBalance {
    pub asset: String,
    pub total: String,
    pub available: String,
    pub locked: String,
}
