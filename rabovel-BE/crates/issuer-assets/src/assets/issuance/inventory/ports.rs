use crate::assets::drafts::models::AssetDraftRecord;
use crate::assets::issuance::inventory::models::InitialInventory;
use crate::assets::issuance::setup::models::AssetSetupOperation;
use crate::assets::issuance::setup::models::SetupExecutionError;
use async_trait::async_trait;

#[async_trait]
pub trait InventoryProvider: Send + Sync {
    async fn initial_inventory(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
        issue: bool,
    ) -> Result<InitialInventory, SetupExecutionError>;
}
