use std::sync::Arc;
pub(crate) trait AssetStore {
    fn assets(&self) -> &Arc<dyn issuer_assets::assets::drafts::ports::AssetRepository>;
}
