use crate::features::settlement::direct_purchases::providers::solana::DisabledPurchaseProvider;
use issuer_assets::assets::issuance::inventory::ports::InventoryProvider;
use issuer_assets::assets::issuance::setup::ports::MintSetupExecutor;
use issuer_assets::assets::positions::ports::PositionProvider;
use settlement_orchestrator::direct_purchases::preparation::ports::PurchasePreparation;
use settlement_orchestrator::direct_purchases::submission::ports::PurchaseSubmission;
use std::sync::Arc;

#[derive(Clone)]
pub struct ChainProviders {
    pub mint_setup: Arc<dyn MintSetupExecutor>,
    pub inventory: Arc<dyn InventoryProvider>,
    pub positions: Arc<dyn PositionProvider>,
    pub purchase_preparation: Arc<dyn PurchasePreparation>,
    pub purchase_submission: Arc<dyn PurchaseSubmission>,
}
impl ChainProviders {
    pub fn from_shared<
        T: MintSetupExecutor
            + InventoryProvider
            + PositionProvider
            + PurchasePreparation
            + PurchaseSubmission
            + 'static,
    >(
        provider: Arc<T>,
    ) -> Self {
        Self {
            mint_setup: provider.clone(),
            inventory: provider.clone(),
            positions: provider.clone(),
            purchase_preparation: provider.clone(),
            purchase_submission: provider.clone(),
        }
    }
    pub(crate) fn disabled() -> Self {
        let issuance = Arc::new(
            crate::features::issuer::assets::issuance::providers::solana::DisabledMintSetupExecutor,
        );
        let purchase = Arc::new(DisabledPurchaseProvider);
        Self {
            mint_setup: issuance.clone(),
            inventory: issuance.clone(),
            positions: issuance,
            purchase_preparation: purchase.clone(),
            purchase_submission: purchase,
        }
    }
}
