use std::sync::Arc;
pub(crate) trait PaymentProvider {
    fn payment_assets(
        &self,
    ) -> &Arc<dyn crate::features::payments::cngn::ports::PaymentAssetRegistry>;
}

use super::models::PaymentAssetStatus;

use super::models::WalletPaymentAssetStatus;

use async_trait::async_trait;

#[async_trait]
pub trait PaymentAssetRegistry: Send + Sync {
    async fn cngn_status(&self) -> PaymentAssetStatus;
    async fn ensure_cngn_broker_account(&self) -> Result<PaymentAssetStatus, String> {
        Err("CNGN broker account creation is not configured".into())
    }
    async fn cngn_wallet_status(&self, _owner: &str) -> Result<WalletPaymentAssetStatus, String> {
        Err("CNGN wallet accounts are not configured".into())
    }
    async fn ensure_cngn_wallet_account(
        &self,
        _owner: &str,
    ) -> Result<WalletPaymentAssetStatus, String> {
        Err("CNGN wallet account creation is not configured".into())
    }
}
