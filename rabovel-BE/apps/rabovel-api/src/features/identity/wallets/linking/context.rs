use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    accounts: Arc<dyn identity::accounts::ports::AccountRepository>,
    auth_service: Arc<identity::BrokerageAuthService>,
    challenges: Arc<dyn identity::wallets::linking::ports::ChallengeRepository>,
    event_producer: Arc<dyn events::EventProducer>,
    profile_updates: Arc<dyn identity::accounts::ports::ProfileUpdates>,
    public_origin: Arc<str>,
    siwe_domain: Arc<str>,
    wallet_proof: Arc<dyn identity::wallets::linking::ports::WalletProofVerifier>,
    wallets: Arc<dyn identity::wallets::ports::WalletRepository>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            accounts: state.accounts.clone(),
            auth_service: state.auth_service.clone(),
            challenges: state.challenges.clone(),
            event_producer: state.event_producer.clone(),
            profile_updates: state.profile_updates.clone(),
            public_origin: state.public_origin.clone(),
            siwe_domain: state.siwe_domain.clone(),
            wallet_proof: state.wallet_proof.clone(),
            wallets: state.wallets.clone(),
        }
    }
}
impl crate::features::identity::accounts::ports::AccountLookup for FeatureContext {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries> {
        &self.account_queries
    }
}
impl crate::features::identity::accounts::ports::AccountStore for FeatureContext {
    fn accounts(&self) -> &Arc<dyn identity::accounts::ports::AccountRepository> {
        &self.accounts
    }
}
impl crate::features::identity::access::ports::AccessPolicy for FeatureContext {
    fn auth_service(&self) -> &Arc<identity::BrokerageAuthService> {
        &self.auth_service
    }
}
impl crate::features::identity::wallets::linking::ports::ChallengeStore for FeatureContext {
    fn challenges(&self) -> &Arc<dyn identity::wallets::linking::ports::ChallengeRepository> {
        &self.challenges
    }
}
impl crate::features::compliance::onboarding::ports::EventPublisher for FeatureContext {
    fn event_producer(&self) -> &Arc<dyn events::EventProducer> {
        &self.event_producer
    }
}
impl crate::features::identity::accounts::ports::ProfileUpdates for FeatureContext {
    fn profile_updates(&self) -> &Arc<dyn identity::accounts::ports::ProfileUpdates> {
        &self.profile_updates
    }
}
impl crate::features::identity::wallets::linking::ports::WalletOrigin for FeatureContext {
    fn public_origin(&self) -> &Arc<str> {
        &self.public_origin
    }
}
impl crate::features::identity::wallets::linking::ports::WalletDomain for FeatureContext {
    fn siwe_domain(&self) -> &Arc<str> {
        &self.siwe_domain
    }
}
impl crate::features::identity::wallets::linking::ports::WalletProof for FeatureContext {
    fn wallet_proof(&self) -> &Arc<dyn identity::wallets::linking::ports::WalletProofVerifier> {
        &self.wallet_proof
    }
}
impl crate::features::identity::wallets::queries::ports::WalletStore for FeatureContext {
    fn wallets(&self) -> &Arc<dyn identity::wallets::ports::WalletRepository> {
        &self.wallets
    }
}
