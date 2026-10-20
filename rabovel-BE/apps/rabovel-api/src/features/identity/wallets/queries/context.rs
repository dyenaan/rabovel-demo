use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    wallet_queries: Arc<dyn identity::wallets::ports::WalletQueries>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            wallet_queries: state.wallet_queries.clone(),
        }
    }
}
impl crate::features::identity::accounts::ports::AccountLookup for FeatureContext {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries> {
        &self.account_queries
    }
}
impl crate::features::identity::wallets::queries::ports::WalletLookup for FeatureContext {
    fn wallet_queries(&self) -> &Arc<dyn identity::wallets::ports::WalletQueries> {
        &self.wallet_queries
    }
}
