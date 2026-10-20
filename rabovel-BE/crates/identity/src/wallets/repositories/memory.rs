use super::super::ports::WalletRepository;
use crate::AuthError;
use crate::WalletConnection;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("identity repository unavailable".into())
}
#[derive(Default)]
pub struct InMemoryWallets {
    wallets: RwLock<HashMap<String, WalletConnection>>,
}
#[async_trait]
impl WalletRepository for InMemoryWallets {
    async fn insert_wallet(&self, wallet: WalletConnection) -> Result<(), AuthError> {
        self.wallets
            .write()
            .map_err(|_| state_unavailable())?
            .insert(wallet.wallet_id().to_string(), wallet);
        Ok(())
    }
}
#[async_trait::async_trait]
impl super::super::ports::WalletQueries for InMemoryWallets {
    async fn get_wallet(&self, wallet_id: &str) -> Result<Option<WalletConnection>, AuthError> {
        Ok(self
            .wallets
            .read()
            .map_err(|_| state_unavailable())?
            .get(wallet_id)
            .cloned())
    }
    async fn list_wallets(&self, owner_user_id: &str) -> Result<Vec<WalletConnection>, AuthError> {
        Ok(self
            .wallets
            .read()
            .map_err(|_| state_unavailable())?
            .values()
            .filter(|wallet| wallet.owner_user_id() == owner_user_id)
            .cloned()
            .collect())
    }
}
