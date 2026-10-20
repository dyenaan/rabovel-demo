use std::sync::Arc;
pub(crate) trait MetadataProvider {
    fn metadata_storage(&self)
        -> &Arc<dyn issuer_assets::assets::metadata::ports::MetadataStorage>;
}
