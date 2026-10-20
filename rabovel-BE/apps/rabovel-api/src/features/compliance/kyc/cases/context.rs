use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    cases: Arc<dyn kyc::cases::ports::CaseRepository>,
    event_producer: Arc<dyn events::EventProducer>,
    kyc_provider: Arc<dyn kyc::KycProviderApi>,
    rate_limiter: Arc<dyn identity::access::rate_limiter::RateLimiter>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            cases: state.cases.clone(),
            event_producer: state.event_producer.clone(),
            kyc_provider: state.kyc_provider.clone(),
            rate_limiter: state.rate_limiter.clone(),
        }
    }
}
impl crate::features::identity::accounts::ports::AccountLookup for FeatureContext {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries> {
        &self.account_queries
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
impl crate::features::identity::access::ports::RateLimit for FeatureContext {
    fn rate_limiter(&self) -> &Arc<dyn identity::access::rate_limiter::RateLimiter> {
        &self.rate_limiter
    }
}
