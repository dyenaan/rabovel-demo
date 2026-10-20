use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    account_queries: Arc<dyn identity::accounts::ports::AccountQueries>,
    assets: Arc<dyn issuer_assets::assets::drafts::ports::AssetRepository>,
    mint_setup: Arc<dyn issuer_assets::assets::issuance::setup::ports::MintSetupExecutor>,
    organizations: Arc<dyn issuer_assets::organizations::ports::OrganizationRepository>,
    setups: Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupRepository>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            account_queries: state.account_queries.clone(),
            assets: state.assets.clone(),
            mint_setup: state.mint_setup.clone(),
            organizations: state.organizations.clone(),
            setups: state.setups.clone(),
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
impl crate::features::issuer::assets::issuance::setup::ports::IssuanceProvider for FeatureContext {
    fn mint_setup(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::setup::ports::MintSetupExecutor> {
        &self.mint_setup
    }
}
impl crate::features::issuer::organizations::ports::OrganizationStore for FeatureContext {
    fn organizations(
        &self,
    ) -> &Arc<dyn issuer_assets::organizations::ports::OrganizationRepository> {
        &self.organizations
    }
}
impl crate::features::issuer::assets::issuance::setup::ports::SetupStore for FeatureContext {
    fn setups(&self) -> &Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupRepository> {
        &self.setups
    }
}
