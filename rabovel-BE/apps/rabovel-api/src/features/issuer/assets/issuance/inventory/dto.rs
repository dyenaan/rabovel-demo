use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct InitialInventoryResponse {
    pub network: String,
    pub mint_address: String,
    pub settlement_wallet: String,
    pub token_account: String,
    pub authorized_units: String,
    pub supply: String,
    pub inventory_balance: String,
    pub wallet_allowlisted: bool,
    pub token_account_ready: bool,
    pub issuance_complete: bool,
    pub signatures: Vec<String>,
}
