use std::sync::Arc;
pub(crate) trait OrganizationStore {
    fn organizations(
        &self,
    ) -> &Arc<dyn issuer_assets::organizations::ports::OrganizationRepository>;
}
