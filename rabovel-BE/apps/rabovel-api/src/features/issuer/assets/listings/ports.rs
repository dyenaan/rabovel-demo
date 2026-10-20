use std::sync::Arc;
pub(crate) trait PublishedAssets {
    fn asset_queries(&self) -> &Arc<dyn issuer_assets::assets::drafts::ports::AssetQueries>;
}
