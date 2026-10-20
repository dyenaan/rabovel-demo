use std::sync::Arc;
pub(crate) trait Inventory {
    fn inventory(
        &self,
    ) -> &Arc<dyn issuer_assets::assets::issuance::inventory::ports::InventoryProvider>;
}
pub(crate) trait Positions {
    fn positions(&self) -> &Arc<dyn issuer_assets::assets::positions::ports::PositionProvider>;
}
