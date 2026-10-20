use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    assets: Arc<dyn issuer_assets::assets::drafts::ports::AssetRepository>,
    organizations: Arc<dyn issuer_assets::organizations::ports::OrganizationRepository>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            assets: state.assets.clone(),
            organizations: state.organizations.clone(),
        }
    }
}
impl crate::features::identity::accounts::ports::AccountLookup for FeatureContext {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries> {
        &self.account_queries
    }
}
impl crate::features::issuer::assets::drafts::ports::AssetStore for FeatureContext {
    fn assets(&self) -> &Arc<dyn issuer_assets::assets::drafts::ports::AssetRepository> {
        &self.assets
    }
}
impl crate::features::issuer::organizations::ports::OrganizationStore for FeatureContext {
    fn organizations(
        &self,
    ) -> &Arc<dyn issuer_assets::organizations::ports::OrganizationRepository> {
        &self.organizations
    }
}
