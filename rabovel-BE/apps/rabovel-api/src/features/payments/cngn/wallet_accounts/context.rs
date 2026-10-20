use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    payment_assets: Arc<dyn crate::features::payments::cngn::ports::PaymentAssetRegistry>,
    rate_limiter: Arc<dyn identity::access::rate_limiter::RateLimiter>,
    wallet_queries: Arc<dyn identity::wallets::ports::WalletQueries>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            payment_assets: state.payment_assets.clone(),
            rate_limiter: state.rate_limiter.clone(),
            wallet_queries: state.wallet_queries.clone(),
        }
    }
}
impl crate::features::identity::accounts::ports::AccountLookup for FeatureContext {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries> {
        &self.account_queries
    }
}
impl crate::features::payments::cngn::ports::PaymentProvider for FeatureContext {
    fn payment_assets(
        &self,
    ) -> &Arc<dyn crate::features::payments::cngn::ports::PaymentAssetRegistry> {
        &self.payment_assets
    }
}
impl crate::features::identity::access::ports::RateLimit for FeatureContext {
    fn rate_limiter(&self) -> &Arc<dyn identity::access::rate_limiter::RateLimiter> {
        &self.rate_limiter
    }
}
impl crate::features::identity::wallets::queries::ports::WalletLookup for FeatureContext {
    fn wallet_queries(&self) -> &Arc<dyn identity::wallets::ports::WalletQueries> {
        &self.wallet_queries
    }
}
