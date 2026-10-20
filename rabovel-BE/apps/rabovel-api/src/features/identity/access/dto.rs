use identity::WalletAction;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WalletAuthorizationRequest {
    pub wallet_id: String,
    pub action: WalletAction,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletAuthorizationResponse {
    pub authorized: bool,
}
