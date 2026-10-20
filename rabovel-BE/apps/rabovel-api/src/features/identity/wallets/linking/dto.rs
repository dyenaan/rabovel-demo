use identity::BrokerageOnboardingStatus;
use identity::WalletConnection;
use identity::WalletProvider;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WalletChallengeRequest {
    pub address: String,
    pub chain_id: u64,
    pub provider: WalletProvider,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SolanaWalletChallengeRequest {
    pub address: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletChallengeResponse {
    pub challenge_id: String,
    pub message: String,
    pub expires_at: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WalletLinkRequest {
    pub challenge_id: String,
    pub signature: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletLinkResponse {
    pub wallet: WalletConnection,
    pub onboarding_status: BrokerageOnboardingStatus,
}
