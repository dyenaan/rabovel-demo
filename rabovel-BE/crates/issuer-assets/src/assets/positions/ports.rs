use crate::assets::issuance::setup::models::AssetSetupOperation;
use crate::assets::issuance::setup::models::SetupExecutionError;
use crate::assets::positions::models::InvestorAssetPosition;
use async_trait::async_trait;

#[async_trait]
pub trait PositionProvider: Send + Sync {
    async fn investor_position(
        &self,
        operation: &AssetSetupOperation,
        wallet: &str,
    ) -> Result<InvestorAssetPosition, SetupExecutionError>;
}
