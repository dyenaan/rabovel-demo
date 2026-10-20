use super::models::WalletChallengeRecord;
use crate::AuthError;
use async_trait::async_trait;

#[async_trait]
pub trait ChallengeRepository: Send + Sync {
    async fn put_wallet_challenge(
        &self,
        challenge_id: &str,
        challenge: WalletChallengeRecord,
        ttl_secs: u64,
    ) -> Result<(), AuthError>;
    async fn take_wallet_challenge(
        &self,
        challenge_id: &str,
    ) -> Result<Option<WalletChallengeRecord>, AuthError>;
    async fn is_wallet_challenge_used(&self, challenge_id: &str) -> Result<bool, AuthError>;

    async fn mark_wallet_challenge_used(
        &self,
        challenge_id: &str,
        ttl_secs: u64,
    ) -> Result<(), AuthError>;
}

pub trait WalletProofVerifier: Send + Sync {
    fn verify(&self, message: &str, signature: &str) -> Result<String, AuthError>;
}
