use crate::SupportedChain;
use crate::WalletProvider;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletChallengeRecord {
    pub user_id: String,
    pub expected_address: String,
    pub chain: SupportedChain,
    pub provider: WalletProvider,
    pub message: String,
}
