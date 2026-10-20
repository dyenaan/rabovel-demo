use crate::features::payments::cngn::models::*;
use crate::features::payments::cngn::ports::PaymentAssetRegistry;
use async_trait::async_trait;

pub struct DisabledPaymentAssetRegistry;

#[async_trait]
impl PaymentAssetRegistry for DisabledPaymentAssetRegistry {
    async fn cngn_status(&self) -> PaymentAssetStatus {
        PaymentAssetStatus {
            code: "CNGN".into(),
            name: "cNGN".into(),
            network: "unconfigured".into(),
            mint_address: None,
            token_program: None,
            decimals: None,
            supply_base_units: None,
            broker_owner_address: None,
            broker_token_account: None,
            broker_balance_base_units: None,
            fee_payer_address: None,
            mint_verified: false,
            broker_account_verified: false,
            ready: false,
            error: Some("CNGN payment asset is not configured".into()),
        }
    }
}
