use std::sync::Arc;
pub(crate) trait WalletStore {
    fn wallets(&self) -> &Arc<dyn identity::wallets::ports::WalletRepository>;
}
pub(crate) trait WalletLookup {
    fn wallet_queries(&self) -> &Arc<dyn identity::wallets::ports::WalletQueries>;
}
