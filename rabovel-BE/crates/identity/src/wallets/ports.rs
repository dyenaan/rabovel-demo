use crate::AuthError;
use crate::WalletConnection;
use async_trait::async_trait;

#[async_trait]
pub trait WalletRepository: WalletQueries + Send + Sync {
    async fn insert_wallet(&self, wallet: WalletConnection) -> Result<(), AuthError>;
}

#[async_trait::async_trait]
pub trait WalletQueries: Send + Sync {
    async fn get_wallet(&self, wallet_id: &str) -> Result<Option<WalletConnection>, AuthError>;

    async fn list_wallets(&self, owner_user_id: &str) -> Result<Vec<WalletConnection>, AuthError>;
}
