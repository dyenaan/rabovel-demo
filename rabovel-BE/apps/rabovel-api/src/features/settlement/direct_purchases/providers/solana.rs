use async_trait::async_trait;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::issuance::setup::models::AssetSetupOperation;
use settlement_orchestrator::direct_purchases::errors::PurchaseExecutionError;
use settlement_orchestrator::direct_purchases::preparation::models::PurchaseTerms;
use settlement_orchestrator::direct_purchases::preparation::ports::PurchasePreparation;
use settlement_orchestrator::direct_purchases::submission::ports::PurchaseSubmission;
use solana_adapter::authority_config::SignerConfig;
use solana_adapter::token_acl::AclDeploymentConfig;
use solana_adapter::token_acl::TokenAclBuilder;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::AccountMeta;
use solana_instruction::Instruction;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_associated_token_account_interface::instruction::create_associated_token_account_idempotent;
use std::sync::Arc;

pub(crate) struct SolanaPurchaseProvider {
    rpc_url: String,
    signer_config: Arc<SignerConfig>,
    acl_config: Arc<AclDeploymentConfig>,
}
impl SolanaPurchaseProvider {
    pub(crate) fn new(config: (String, Arc<SignerConfig>, Arc<AclDeploymentConfig>)) -> Self {
        Self {
            rpc_url: config.0,
            signer_config: config.1,
            acl_config: config.2,
        }
    }
}
pub(crate) struct DisabledPurchaseProvider;
#[async_trait]
impl PurchasePreparation for SolanaPurchaseProvider {
    async fn prepare_purchase(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
        terms: PurchaseTerms<'_>,
    ) -> Result<Vec<u8>, PurchaseExecutionError> {
        let run = async {
            let signers = self
                .signer_config
                .load(&asset.issuer_user_id)
                .map_err(|error| error.to_string())?;
            let rpc =
                RpcClient::new_with_commitment(self.rpc_url.clone(), CommitmentConfig::confirmed());
            let investor = terms
                .investor_wallet
                .parse::<Pubkey>()
                .map_err(|_| "linked Solana wallet is invalid".to_string())?;
            let mint = operation
                .mint_address
                .parse::<Pubkey>()
                .map_err(|_| "stored mint address is invalid".to_string())?;
            let investor_equity = get_associated_token_address_with_program_id(
                &investor,
                &mint,
                &spl_token_2022::id(),
            );
            let admin = signers.rabovel_admin.pubkey();
            let plan = self
                .acl_config
                .plan(admin)
                .map_err(|error| error.to_string())?;
            let wallet_entry = Pubkey::find_program_address(
                &[
                    b"wallet_entry",
                    plan.addresses.allow.as_ref(),
                    investor.as_ref(),
                ],
                &plan.gate_program,
            )
            .0;
            let mut setup = Vec::new();
            if rpc
                .get_account_with_commitment(&wallet_entry, CommitmentConfig::confirmed())
                .await
                .map_err(|error| format!("could not inspect investor allow-list entry: {error}"))?
                .value
                .is_none()
            {
                setup.push(Instruction {
                    program_id: plan.gate_program,
                    accounts: vec![
                        AccountMeta::new_readonly(admin, true),
                        AccountMeta::new(admin, true),
                        AccountMeta::new(plan.addresses.allow, false),
                        AccountMeta::new_readonly(investor, false),
                        AccountMeta::new(wallet_entry, false),
                        AccountMeta::new_readonly(solana_system_interface::program::id(), false),
                    ],
                    data: vec![2],
                });
            }
            if rpc
                .get_account_with_commitment(&investor_equity, CommitmentConfig::confirmed())
                .await
                .map_err(|error| format!("could not inspect investor equity account: {error}"))?
                .value
                .is_none()
            {
                setup.push(create_associated_token_account_idempotent(
                    &admin,
                    &investor,
                    &mint,
                    &spl_token_2022::id(),
                ));
            }
            if !setup.is_empty() {
                solana_adapter::accounts::submit(&rpc, setup, &signers.rabovel_admin, None).await?;
            }
            if !solana_adapter::accounts::token_account_is_initialized(
                &rpc,
                investor_equity,
                mint,
                investor,
            )
            .await?
            {
                let mint_config = TokenAclBuilder::mint_config_address(&mint);
                let rpc_ref = &rpc;
                let fetch = move |address: Pubkey| async move {
                    rpc_ref
                        .get_account_with_commitment(&address, CommitmentConfig::confirmed())
                        .await
                        .map(|response| response.value.map(|account| account.data))
                        .map_err(Into::into)
                };
                let thaw =
                    token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
                        &admin,
                        &investor_equity,
                        &mint,
                        &mint_config,
                        &spl_token_2022::id(),
                        &investor,
                        true,
                        fetch,
                    )
                    .await
                    .map_err(|error| format!("could not prepare investor account thaw: {error}"))?;
                solana_adapter::accounts::submit(&rpc, vec![thaw], &signers.rabovel_admin, None)
                    .await?;
            }

            let payment_program = terms
                .payment_token_program
                .parse::<Pubkey>()
                .map_err(|_| "payment token program is invalid".to_string())?;
            let payment_mint = terms
                .payment_mint
                .parse::<Pubkey>()
                .map_err(|_| "payment mint is invalid".to_string())?;
            let investor_payment = terms
                .investor_payment_account
                .parse::<Pubkey>()
                .map_err(|_| "investor payment account is invalid".to_string())?;
            let broker_payment = terms
                .broker_payment_account
                .parse::<Pubkey>()
                .map_err(|_| "broker payment account is invalid".to_string())?;
            let payment_transfer = spl_token_2022::instruction::transfer_checked(
                &payment_program,
                &investor_payment,
                &payment_mint,
                &broker_payment,
                &investor,
                &[],
                terms.payment_amount,
                terms.payment_decimals,
            )
            .map_err(|error| format!("could not prepare cNGN transfer: {error}"))?;

            let issuer_equity = get_associated_token_address_with_program_id(
                &signers.issuer.pubkey(),
                &mint,
                &spl_token_2022::id(),
            );
            let rpc_ref = &rpc;
            let fetch = move |address: Pubkey| async move {
                rpc_ref
                    .get_account_with_commitment(&address, CommitmentConfig::confirmed())
                    .await
                    .map(|response| response.value.map(|account| account.data))
                    .map_err(Into::into)
            };
            let equity_transfer =
                spl_token_2022::offchain::create_transfer_checked_instruction_with_extra_metas(
                    &spl_token_2022::id(),
                    &issuer_equity,
                    &mint,
                    &investor_equity,
                    &signers.issuer.pubkey(),
                    &[],
                    terms.equity_quantity,
                    0,
                    fetch,
                )
                .await
                .map_err(|error| format!("could not prepare equity transfer: {error}"))?;
            let blockhash = rpc
                .get_latest_blockhash()
                .await
                .map_err(|error| format!("could not get a recent blockhash: {error}"))?;
            let mut transaction = Transaction::new_unsigned(Message::new(
                &[payment_transfer, equity_transfer],
                Some(&investor),
            ));
            transaction
                .try_partial_sign(&[&signers.issuer], blockhash)
                .map_err(|error| format!("could not partially sign purchase: {error}"))?;
            bincode::serialize(&transaction)
                .map_err(|error| format!("could not serialize purchase transaction: {error}"))
        }
        .await;
        run.map_err(|message| PurchaseExecutionError {
            reconciliation_required: false,
            message,
        })
    }
}
#[async_trait]
impl PurchasePreparation for DisabledPurchaseProvider {
    async fn prepare_purchase(
        &self,
        _: &AssetDraftRecord,
        _: &AssetSetupOperation,
        _: PurchaseTerms<'_>,
    ) -> Result<Vec<u8>, PurchaseExecutionError> {
        Err(PurchaseExecutionError {
            message: "Solana purchase settlement is not configured".into(),
            reconciliation_required: false,
        })
    }
}
#[async_trait]
impl PurchaseSubmission for SolanaPurchaseProvider {
    async fn submit_purchase(
        &self,
        investor_wallet: &str,
        transaction: &[u8],
    ) -> Result<String, PurchaseExecutionError> {
        let run = async {
            let transaction: Transaction = bincode::deserialize(transaction)
                .map_err(|_| "signed purchase transaction is invalid".to_string())?;
            let investor = investor_wallet
                .parse::<Pubkey>()
                .map_err(|_| "linked Solana wallet is invalid".to_string())?;
            if transaction.message.account_keys.first() != Some(&investor) {
                return Err(
                    "purchase transaction fee payer does not match the linked wallet".into(),
                );
            }
            transaction
                .verify()
                .map_err(|_| "purchase transaction signatures are invalid".to_string())?;
            let rpc =
                RpcClient::new_with_commitment(self.rpc_url.clone(), CommitmentConfig::confirmed());
            rpc.send_and_confirm_transaction(&transaction)
                .await
                .map(|signature| signature.to_string())
                .map_err(|error| format!("purchase transaction was not confirmed: {error}"))
        }
        .await;
        run.map_err(|message| PurchaseExecutionError {
            reconciliation_required: message.contains("not confirmed"),
            message,
        })
    }
}
#[async_trait]
impl PurchaseSubmission for DisabledPurchaseProvider {
    async fn submit_purchase(&self, _: &str, _: &[u8]) -> Result<String, PurchaseExecutionError> {
        Err(PurchaseExecutionError {
            message: "Solana purchase settlement is not configured".into(),
            reconciliation_required: false,
        })
    }
}
