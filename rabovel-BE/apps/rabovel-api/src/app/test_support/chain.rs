use crate::features::payments::cngn::models::PaymentAssetStatus;
use crate::features::payments::cngn::models::WalletPaymentAssetStatus;
use crate::features::payments::cngn::ports::PaymentAssetRegistry;
use async_trait::async_trait;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::issuance::inventory::models::InitialInventory;
use issuer_assets::assets::issuance::setup::models::AssetSetupOperation;
use issuer_assets::assets::issuance::setup::models::SetupExecution;
use issuer_assets::assets::issuance::setup::models::SetupExecutionError;
use issuer_assets::assets::positions::models::InvestorAssetPosition;
use settlement_orchestrator::direct_purchases::errors::PurchaseExecutionError;
use settlement_orchestrator::direct_purchases::preparation::models::PurchaseTerms;

#[derive(Default)]
pub(crate) struct FakeMintSetup {
    issued: std::sync::atomic::AtomicBool,
}
pub(crate) struct FakePaymentAssets;

#[async_trait]
impl PaymentAssetRegistry for FakePaymentAssets {
    async fn cngn_status(&self) -> PaymentAssetStatus {
        PaymentAssetStatus {
            code: "CNGN".into(),
            name: "cNGN".into(),
            network: "testnet".into(),
            mint_address: Some("test-mint".into()),
            token_program: Some("spl-token-2022".into()),
            decimals: Some(6),
            supply_base_units: Some("1000000000".into()),
            broker_owner_address: Some("broker".into()),
            broker_token_account: Some("broker-account".into()),
            broker_balance_base_units: Some("100000000".into()),
            fee_payer_address: Some("admin".into()),
            mint_verified: true,
            broker_account_verified: true,
            ready: true,
            error: None,
        }
    }

    async fn ensure_cngn_broker_account(&self) -> Result<PaymentAssetStatus, String> {
        Ok(self.cngn_status().await)
    }

    async fn cngn_wallet_status(&self, owner: &str) -> Result<WalletPaymentAssetStatus, String> {
        Ok(WalletPaymentAssetStatus {
            code: "CNGN".into(),
            name: "cNGN".into(),
            network: "devnet".into(),
            mint_address: "test-mint".into(),
            token_program: Some("spl-token-2022".into()),
            decimals: Some(6),
            wallet_address: owner.into(),
            token_account: Some("investor-cngn-account".into()),
            balance_base_units: Some("25000000".into()),
            account_verified: true,
            ready: true,
            error: None,
        })
    }

    async fn ensure_cngn_wallet_account(
        &self,
        owner: &str,
    ) -> Result<WalletPaymentAssetStatus, String> {
        self.cngn_wallet_status(owner).await
    }
}
#[async_trait]
impl issuer_assets::assets::issuance::setup::ports::MintSetupExecutor for FakeMintSetup {
    fn is_configured(&self) -> bool {
        true
    }
    fn network(&self) -> &str {
        "devnet"
    }
    fn mint_address(&self, operation_id: &str) -> String {
        format!("mint-{operation_id}")
    }
    async fn execute(
        &self,
        _: &AssetDraftRecord,
        _: &AssetSetupOperation,
    ) -> Result<SetupExecution, SetupExecutionError> {
        Ok(SetupExecution {
            mint_signature: Some("mint-signature".into()),
            shared_lists_signature: Some("lists-signature".into()),
            acl_signatures: vec!["acl-signature".into()],
            mint_config_address: "mint-config".into(),
            allow_list_address: "allow-list".into(),
            block_list_address: "block-list".into(),
            thaw_extra_metas_address: "thaw-extra-metas".into(),
        })
    }
}
#[async_trait]
impl issuer_assets::assets::issuance::inventory::ports::InventoryProvider for FakeMintSetup {
    async fn initial_inventory(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
        issue: bool,
    ) -> Result<InitialInventory, SetupExecutionError> {
        if issue {
            self.issued.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        let issued = self.issued.load(std::sync::atomic::Ordering::SeqCst);
        let amount = if issued {
            asset.draft.authorized_units.to_string()
        } else {
            "0".into()
        };
        Ok(InitialInventory {
            network: operation.network.clone(),
            mint_address: operation.mint_address.clone(),
            settlement_wallet: "issuer-settlement-wallet".into(),
            token_account: "issuer-inventory-account".into(),
            authorized_units: asset.draft.authorized_units.to_string(),
            supply: amount.clone(),
            inventory_balance: amount,
            wallet_allowlisted: issued,
            token_account_ready: issued,
            issuance_complete: issued,
            signatures: issue
                .then(|| "inventory-signature".into())
                .into_iter()
                .collect(),
        })
    }
}
#[async_trait]
impl issuer_assets::assets::positions::ports::PositionProvider for FakeMintSetup {
    async fn investor_position(
        &self,
        _: &AssetSetupOperation,
        _: &str,
    ) -> Result<InvestorAssetPosition, SetupExecutionError> {
        Ok(InvestorAssetPosition {
            token_account: "investor-equity-account".into(),
            balance: "0".into(),
            account_ready: true,
        })
    }
}
#[async_trait]
impl settlement_orchestrator::direct_purchases::preparation::ports::PurchasePreparation
    for FakeMintSetup
{
    async fn prepare_purchase(
        &self,
        _: &AssetDraftRecord,
        _: &AssetSetupOperation,
        _: PurchaseTerms<'_>,
    ) -> Result<Vec<u8>, PurchaseExecutionError> {
        Err(PurchaseExecutionError {
            message: "purchase preparation is not available in the unit fake".into(),
            reconciliation_required: false,
        })
    }
}
#[async_trait]
impl settlement_orchestrator::direct_purchases::submission::ports::PurchaseSubmission
    for FakeMintSetup
{
    async fn submit_purchase(&self, _: &str, _: &[u8]) -> Result<String, PurchaseExecutionError> {
        Err(PurchaseExecutionError {
            message: "purchase submission is not available in the unit fake".into(),
            reconciliation_required: false,
        })
    }
}
