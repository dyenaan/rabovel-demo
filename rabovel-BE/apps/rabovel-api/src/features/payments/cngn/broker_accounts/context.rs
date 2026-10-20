use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    organizations: Arc<dyn issuer_assets::organizations::ports::OrganizationRepository>,
    payment_assets: Arc<dyn crate::features::payments::cngn::ports::PaymentAssetRegistry>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            organizations: state.organizations.clone(),
            payment_assets: state.payment_assets.clone(),
        }
    }
}
impl crate::features::identity::accounts::ports::AccountLookup for FeatureContext {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries> {
        &self.account_queries
    }
}
impl crate::features::issuer::organizations::ports::OrganizationStore for FeatureContext {
    fn organizations(
        &self,
    ) -> &Arc<dyn issuer_assets::organizations::ports::OrganizationRepository> {
        &self.organizations
    }
}
impl crate::features::payments::cngn::ports::PaymentProvider for FeatureContext {
    fn payment_assets(
        &self,
    ) -> &Arc<dyn crate::features::payments::cngn::ports::PaymentAssetRegistry> {
        &self.payment_assets
    }
}
