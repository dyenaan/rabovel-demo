use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    accounts: Arc<dyn identity::accounts::ports::AccountRepository>,
    auth_service: Arc<identity::BrokerageAuthService>,
    sessions: Arc<dyn identity::sessions::ports::SessionRepository>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            accounts: state.accounts.clone(),
            auth_service: state.auth_service.clone(),
            sessions: state.sessions.clone(),
        }
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
impl crate::features::identity::sessions::ports::SessionStore for FeatureContext {
    fn sessions(&self) -> &Arc<dyn identity::sessions::ports::SessionRepository> {
        &self.sessions
    }
}
