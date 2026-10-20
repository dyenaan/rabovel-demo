use crate::assets::drafts::models::AssetDraftRecord;
use crate::assets::issuance::setup::models::AssetSetupOperation;
use crate::assets::issuance::setup::models::SetupExecution;
use crate::assets::issuance::setup::models::SetupExecutionError;
use async_trait::async_trait;
use identity::AuthError;

#[async_trait]
pub trait SetupRepository: SetupQueries + Send + Sync {
    async fn create_asset_setup_operation(
        &self,
        operation: AssetSetupOperation,
    ) -> Result<AssetSetupOperation, AuthError>;
    async fn save_asset_setup_operation(
        &self,
        operation: AssetSetupOperation,
    ) -> Result<(), AuthError>;
}

#[async_trait]
pub trait MintSetupExecutor: Send + Sync {
    fn is_configured(&self) -> bool;

    fn network(&self) -> &str;

    fn mint_address(&self, operation_id: &str) -> String;

    async fn execute(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
    ) -> Result<SetupExecution, SetupExecutionError>;
}

#[async_trait::async_trait]
pub trait SetupQueries: Send + Sync {
    async fn get_asset_setup_operation(
        &self,
        asset_id: &str,
    ) -> Result<Option<AssetSetupOperation>, AuthError>;
}
