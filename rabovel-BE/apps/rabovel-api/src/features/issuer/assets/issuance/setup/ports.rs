use std::sync::Arc;
pub(crate) trait SetupStore {
    fn setups(&self) -> &Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupRepository>;
}
pub(crate) trait IssuanceProvider {
    fn mint_setup(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::setup::ports::MintSetupExecutor>;
}
pub(crate) trait SetupLookup {
    fn setup_queries(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::setup::ports::SetupQueries>;
}
