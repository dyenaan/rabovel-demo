#[derive(Debug, Clone)]
pub struct InvestorAssetPosition {
    pub token_account: String,
    pub balance: String,
    pub account_ready: bool,
}
