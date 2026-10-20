use std::sync::Arc;
pub(crate) trait Submission {
    fn purchase_submission(
        &self,
    ) -> &Arc<dyn settlement_orchestrator::direct_purchases::submission::ports::PurchaseSubmission>;
}
