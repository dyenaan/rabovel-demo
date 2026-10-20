use crate::direct_purchases::errors::PurchaseExecutionError;
use async_trait::async_trait;

#[async_trait]
pub trait PurchaseSubmission: Send + Sync {
    async fn submit_purchase(
        &self,
        investor_wallet: &str,
        transaction: &[u8],
    ) -> Result<String, PurchaseExecutionError>;
}
