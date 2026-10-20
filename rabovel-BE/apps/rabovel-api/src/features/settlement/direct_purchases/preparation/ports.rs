use std::sync::Arc;
pub(crate) trait Preparation {
    fn purchase_preparation(
        &self,
    ) -> &Arc<dyn settlement_orchestrator::direct_purchases::preparation::ports::PurchasePreparation>;
}
