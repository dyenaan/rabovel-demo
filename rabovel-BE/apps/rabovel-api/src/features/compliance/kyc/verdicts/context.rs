use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    auth_service: Arc<identity::BrokerageAuthService>,
    cases: Arc<dyn kyc::cases::ports::CaseRepository>,
    event_producer: Arc<dyn events::EventProducer>,
    kyc_provider: Arc<dyn kyc::KycProviderApi>,
    profile_updates: Arc<dyn identity::accounts::ports::ProfileUpdates>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            auth_service: state.auth_service.clone(),
            cases: state.cases.clone(),
            event_producer: state.event_producer.clone(),
            kyc_provider: state.kyc_provider.clone(),
            profile_updates: state.profile_updates.clone(),
        }
    }
}
impl crate::features::identity::accounts::ports::AccountLookup for FeatureContext {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries> {
        &self.account_queries
    }
}
impl crate::features::identity::access::ports::AccessPolicy for FeatureContext {
    fn auth_service(&self) -> &Arc<identity::BrokerageAuthService> {
        &self.auth_service
    }
}
impl crate::features::compliance::kyc::cases::ports::CaseStore for FeatureContext {
    fn cases(&self) -> &Arc<dyn kyc::cases::ports::CaseRepository> {
        &self.cases
    }
}
impl crate::features::compliance::onboarding::ports::EventPublisher for FeatureContext {
    fn event_producer(&self) -> &Arc<dyn events::EventProducer> {
        &self.event_producer
    }
}
impl crate::features::compliance::kyc::cases::ports::KycProvider for FeatureContext {
    fn kyc_provider(&self) -> &Arc<dyn kyc::KycProviderApi> {
        &self.kyc_provider
    }
}
impl crate::features::identity::accounts::ports::ProfileUpdates for FeatureContext {
    fn profile_updates(&self) -> &Arc<dyn identity::accounts::ports::ProfileUpdates> {
        &self.profile_updates
    }
}
