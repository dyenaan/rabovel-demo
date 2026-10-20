use crate::direct_purchases::errors::PurchaseExecutionError;
use crate::direct_purchases::preparation::models::PurchaseTerms;
use async_trait::async_trait;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::issuance::setup::models::AssetSetupOperation;

#[async_trait]
pub trait PurchasePreparation: Send + Sync {
    async fn prepare_purchase(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
        terms: PurchaseTerms<'_>,
    ) -> Result<Vec<u8>, PurchaseExecutionError>;
}
