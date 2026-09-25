use async_trait::async_trait;
use identity::{AssetDraftRecord, AssetSetupOperation};
use sha2::{Digest, Sha256};
use solana_adapter::{
    authority_config::SignerConfig,
    token_acl::{AclDeploymentConfig, MintAclService, SharedListsService, TokenAclBuilder},
    token_mint_builder::TokenMetadata,
    EquitySetupService,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_associated_token_account_interface::{
    address::get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent,
};
use spl_token_2022::{
    extension::StateWithExtensions,
    state::{Account as TokenAccount, AccountState},
};
use std::sync::Arc;

#[derive(Debug)]
pub struct SetupExecution {
    pub mint_signature: Option<String>,
    pub shared_lists_signature: Option<String>,
    pub acl_signatures: Vec<String>,
    pub mint_config_address: String,
    pub allow_list_address: String,
    pub block_list_address: String,
    pub thaw_extra_metas_address: String,
}

#[derive(Debug)]
pub struct SetupExecutionError {
    pub message: String,
    pub reconciliation_required: bool,
}

#[derive(Debug, Clone)]
pub struct InitialInventory {
    pub network: String,
    pub mint_address: String,
    pub settlement_wallet: String,
    pub token_account: String,
    pub authorized_units: String,
    pub supply: String,
    pub inventory_balance: String,
    pub wallet_allowlisted: bool,
    pub token_account_ready: bool,
    pub issuance_complete: bool,
    pub signatures: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InvestorAssetPosition {
    pub token_account: String,
    pub balance: String,
    pub account_ready: bool,
}

pub struct PurchaseTerms<'a> {
    pub investor_wallet: &'a str,
    pub investor_payment_account: &'a str,
    pub payment_mint: &'a str,
    pub payment_token_program: &'a str,
    pub payment_decimals: u8,
    pub broker_payment_account: &'a str,
    pub equity_quantity: u64,
    pub payment_amount: u64,
}

#[async_trait]
pub trait MintSetupExecutor: Send + Sync {
    fn is_configured(&self) -> bool;
    fn network(&self) -> &str;
    fn mint_address(&self, operation_id: &str) -> String;
    async fn execute(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
    ) -> Result<SetupExecution, SetupExecutionError>;
    async fn initial_inventory(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
        issue: bool,
    ) -> Result<InitialInventory, SetupExecutionError>;
    async fn investor_position(
        &self,
        operation: &AssetSetupOperation,
        wallet: &str,
    ) -> Result<InvestorAssetPosition, SetupExecutionError>;
    async fn prepare_purchase(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
        terms: PurchaseTerms<'_>,
    ) -> Result<Vec<u8>, SetupExecutionError>;
    async fn submit_purchase(
        &self,
        investor_wallet: &str,
        transaction: &[u8],
    ) -> Result<String, SetupExecutionError>;
}

pub struct DisabledMintSetupExecutor;

#[async_trait]
impl MintSetupExecutor for DisabledMintSetupExecutor {
    fn is_configured(&self) -> bool {
        false
    }
    fn network(&self) -> &str {
        "devnet"
    }
    fn mint_address(&self, operation_id: &str) -> String {
        format!("unconfigured-{operation_id}")
    }
    async fn execute(
        &self,
        _: &AssetDraftRecord,
        _: &AssetSetupOperation,
    ) -> Result<SetupExecution, SetupExecutionError> {
        Err(SetupExecutionError {
            message: "Solana mint setup is not configured".into(),
            reconciliation_required: false,
        })
    }
    async fn initial_inventory(
        &self,
        _: &AssetDraftRecord,
        _: &AssetSetupOperation,
        _: bool,
    ) -> Result<InitialInventory, SetupExecutionError> {
        Err(SetupExecutionError {
            message: "Solana inventory issuance is not configured".into(),
            reconciliation_required: false,
        })
    }
    async fn investor_position(
        &self,
        _: &AssetSetupOperation,
        _: &str,
    ) -> Result<InvestorAssetPosition, SetupExecutionError> {
        Err(SetupExecutionError {
            message: "Solana investor balances are not configured".into(),
            reconciliation_required: false,
        })
    }
    async fn prepare_purchase(
        &self,
        _: &AssetDraftRecord,
        _: &AssetSetupOperation,
        _: PurchaseTerms<'_>,
    ) -> Result<Vec<u8>, SetupExecutionError> {
        Err(SetupExecutionError {
            message: "Solana purchase settlement is not configured".into(),
            reconciliation_required: false,
        })
    }
    async fn submit_purchase(&self, _: &str, _: &[u8]) -> Result<String, SetupExecutionError> {
        Err(SetupExecutionError {
            message: "Solana purchase settlement is not configured".into(),
            reconciliation_required: false,
        })
    }
}

pub struct SolanaMintSetupExecutor {
    rpc_url: String,
    network: String,
    signer_config: SignerConfig,
    acl_config: AclDeploymentConfig,
    derivation_secret: Arc<[u8]>,
}

impl SolanaMintSetupExecutor {
    pub fn new(
        rpc_url: String,
        network: String,
        signer_config: SignerConfig,
        acl_config: AclDeploymentConfig,
        derivation_secret: Vec<u8>,
    ) -> Result<Self, String> {
        if !matches!(network.as_str(), "devnet" | "testnet") {
            return Err("RABOVEL_SOLANA_NETWORK must be devnet or testnet".into());
        }
        if derivation_secret.len() < 32 {
            return Err("RABOVEL_MINT_DERIVATION_SECRET must decode to at least 32 bytes".into());
        }
        if acl_config.environment != network {
            return Err("ACL deployment environment does not match RABOVEL_SOLANA_NETWORK".into());
        }
        let admin = signer_config
            .load_admin()
            .map_err(|error| error.to_string())?;
        acl_config
            .plan(admin.pubkey())
            .map_err(|error| error.to_string())?;
        Ok(Self {
            rpc_url,
            network,
            signer_config,
            acl_config,
            derivation_secret: derivation_secret.into(),
        })
    }

    fn mint_keypair(&self, operation_id: &str) -> Keypair {
        let mut hash = Sha256::new();
        hash.update(b"rabovel-mint-setup-v1");
        hash.update(&*self.derivation_secret);
        hash.update(operation_id.as_bytes());
        Keypair::new_from_array(hash.finalize().into())
    }

    async fn submit(
        rpc: &RpcClient,
        instructions: Vec<Instruction>,
        payer: &Keypair,
        additional_signer: Option<&Keypair>,
    ) -> Result<String, String> {
        let blockhash = rpc
            .get_latest_blockhash()
            .await
            .map_err(|error| format!("could not get a recent blockhash: {error}"))?;
        let mut transaction =
            Transaction::new_unsigned(Message::new(&instructions, Some(&payer.pubkey())));
        if additional_signer.is_some_and(|signer| signer.pubkey() != payer.pubkey()) {
            transaction
                .try_sign(&[payer, additional_signer.unwrap()], blockhash)
                .map_err(|error| format!("could not sign inventory transaction: {error}"))?;
        } else {
            transaction
                .try_sign(&[payer], blockhash)
                .map_err(|error| format!("could not sign inventory transaction: {error}"))?;
        }
        rpc.send_and_confirm_transaction(&transaction)
            .await
            .map(|signature| signature.to_string())
            .map_err(|error| format!("inventory transaction was not confirmed: {error}"))
    }

    async fn inventory_snapshot(
        &self,
        rpc: &RpcClient,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
        admin: Pubkey,
        settlement_wallet: Pubkey,
        signatures: Vec<String>,
    ) -> Result<InitialInventory, String> {
        let mint = operation
            .mint_address
            .parse::<Pubkey>()
            .map_err(|_| "stored mint address is invalid".to_string())?;
        let token_account = get_associated_token_address_with_program_id(
            &settlement_wallet,
            &mint,
            &spl_token_2022::id(),
        );
        let plan = self
            .acl_config
            .plan(admin)
            .map_err(|error| error.to_string())?;
        let wallet_entry = Pubkey::find_program_address(
            &[
                b"wallet_entry",
                plan.addresses.allow.as_ref(),
                settlement_wallet.as_ref(),
            ],
            &plan.gate_program,
        )
        .0;
        let wallet_allowlisted = rpc
            .get_account_with_commitment(&wallet_entry, CommitmentConfig::confirmed())
            .await
            .map_err(|error| format!("could not inspect settlement allow-list entry: {error}"))?
            .value
            .is_some();
        let supply = rpc
            .get_token_supply(&mint)
            .await
            .map_err(|error| format!("could not read equity supply: {error}"))?
            .amount;
        let balance = match rpc.get_token_account_balance(&token_account).await {
            Ok(balance) => balance.amount,
            Err(_) => "0".into(),
        };
        let token_account_ready =
            Self::token_account_is_initialized(rpc, token_account, mint, settlement_wallet).await?;
        let authorized = asset.draft.authorized_units.to_string();
        Ok(InitialInventory {
            network: self.network.clone(),
            mint_address: operation.mint_address.clone(),
            settlement_wallet: settlement_wallet.to_string(),
            token_account: token_account.to_string(),
            authorized_units: authorized.clone(),
            // Issuance is a supply invariant, not a transient account-state
            // invariant. ACL-controlled accounts may be frozen or thawed while
            // trading, and sold units leave issuer inventory, but neither event
            // reverses the completed issuance.
            issuance_complete: supply == authorized,
            supply,
            inventory_balance: balance,
            wallet_allowlisted,
            token_account_ready,
            signatures,
        })
    }

    async fn token_account_is_initialized(
        rpc: &RpcClient,
        address: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
    ) -> Result<bool, String> {
        let Some(account) = rpc
            .get_account_with_commitment(&address, CommitmentConfig::confirmed())
            .await
            .map_err(|error| format!("could not inspect settlement token account: {error}"))?
            .value
        else {
            return Ok(false);
        };
        if account.owner != spl_token_2022::id() {
            return Err("settlement token account has an unexpected program owner".into());
        }
        let state = StateWithExtensions::<TokenAccount>::unpack(&account.data)
            .map_err(|error| format!("settlement token account data is invalid: {error}"))?;
        if state.base.mint != mint || state.base.owner != owner {
            return Err("settlement token account mint or owner does not match".into());
        }
        Ok(state.base.state == AccountState::Initialized)
    }
}

#[async_trait]
impl MintSetupExecutor for SolanaMintSetupExecutor {
    fn is_configured(&self) -> bool {
        true
    }
    fn network(&self) -> &str {
        &self.network
    }
    fn mint_address(&self, operation_id: &str) -> String {
        self.mint_keypair(operation_id).pubkey().to_string()
    }

    async fn execute(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
    ) -> Result<SetupExecution, SetupExecutionError> {
        let run = async {
            let signers = self
                .signer_config
                .load(&asset.issuer_user_id)
                .map_err(|error| error.to_string())?;
            let mint = self.mint_keypair(&operation.operation_id);
            if mint.pubkey().to_string() != operation.mint_address {
                return Err(
                    "stored mint address does not match the replay-safe derived signer".into(),
                );
            }
            let metadata_uri = asset
                .draft
                .metadata
                .metadata_uri
                .clone()
                .ok_or_else(|| "hosted metadata URI is required".to_string())?;
            let plan = self
                .acl_config
                .plan(signers.rabovel_admin.pubkey())
                .map_err(|error| error.to_string())?;
            let mut additional_metadata: Vec<_> = asset
                .draft
                .metadata
                .additional_metadata
                .iter()
                .filter(|item| item.key != "token_acl")
                .map(|item| (item.key.clone(), item.value.clone()))
                .collect();
            additional_metadata.push(("token_acl".into(), plan.gate_program.to_string()));
            let metadata = TokenMetadata {
                name: asset.draft.metadata.name.clone(),
                symbol: asset.draft.metadata.symbol.clone(),
                uri: metadata_uri,
                additional_metadata,
            };
            let lists = SharedListsService::new(RpcClient::new_with_commitment(
                self.rpc_url.clone(),
                CommitmentConfig::confirmed(),
            ))
            .create_or_verify(&self.acl_config, &signers.rabovel_admin)
            .await
            .map_err(|error| error.to_string())?;
            let mint_signature = EquitySetupService::new(RpcClient::new_with_commitment(
                self.rpc_url.clone(),
                CommitmentConfig::confirmed(),
            ))
            .create_or_verify_mint(&signers, &mint, metadata)
            .await
            .map_err(|error| error.to_string())?;
            let acl = MintAclService::new(RpcClient::new_with_commitment(
                self.rpc_url.clone(),
                CommitmentConfig::confirmed(),
            ))
            .configure(&self.acl_config, mint.pubkey(), &signers.rabovel_admin)
            .await
            .map_err(|error| error.to_string())?;
            Ok(SetupExecution {
                mint_signature,
                shared_lists_signature: lists.creation_signature,
                acl_signatures: acl.signatures,
                mint_config_address: acl.mint_config.to_string(),
                allow_list_address: lists.addresses.allow.to_string(),
                block_list_address: lists.addresses.block.to_string(),
                thaw_extra_metas_address: acl.thaw_extra_metas.to_string(),
            })
        }
        .await;
        run.map_err(|message: String| SetupExecutionError {
            reconciliation_required: message.contains("transaction")
                || message.contains("confirmed signatures"),
            message,
        })
    }

    async fn initial_inventory(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
        issue: bool,
    ) -> Result<InitialInventory, SetupExecutionError> {
        let run = async {
            if operation.status != "confirmed" || asset.status != "minted" {
                return Err("asset mint setup must be confirmed before issuance".into());
            }
            let signers = self
                .signer_config
                .load(&asset.issuer_user_id)
                .map_err(|error| error.to_string())?;
            let rpc =
                RpcClient::new_with_commitment(self.rpc_url.clone(), CommitmentConfig::confirmed());
            let admin = signers.rabovel_admin.pubkey();
            let settlement_wallet = signers.issuer.pubkey();
            let current = self
                .inventory_snapshot(&rpc, asset, operation, admin, settlement_wallet, vec![])
                .await?;
            if !issue || current.issuance_complete {
                return Ok(current);
            }
            if current.supply != "0" || current.inventory_balance != "0" {
                return Err(
                    "equity already has a conflicting non-zero supply or inventory balance".into(),
                );
            }

            let mint = operation
                .mint_address
                .parse::<Pubkey>()
                .map_err(|_| "stored mint address is invalid".to_string())?;
            let plan = self
                .acl_config
                .plan(admin)
                .map_err(|error| error.to_string())?;
            let wallet_entry = Pubkey::find_program_address(
                &[
                    b"wallet_entry",
                    plan.addresses.allow.as_ref(),
                    settlement_wallet.as_ref(),
                ],
                &plan.gate_program,
            )
            .0;
            let mut signatures = Vec::new();
            let entry = rpc
                .get_account_with_commitment(&wallet_entry, CommitmentConfig::confirmed())
                .await
                .map_err(|error| {
                    format!("could not inspect settlement allow-list entry: {error}")
                })?;
            if entry.value.is_none() {
                let add_wallet = Instruction {
                    program_id: plan.gate_program,
                    accounts: vec![
                        AccountMeta::new_readonly(admin, true),
                        AccountMeta::new(admin, true),
                        AccountMeta::new(plan.addresses.allow, false),
                        AccountMeta::new_readonly(settlement_wallet, false),
                        AccountMeta::new(wallet_entry, false),
                        AccountMeta::new_readonly(solana_system_interface::program::id(), false),
                    ],
                    data: vec![2],
                };
                signatures.push(
                    Self::submit(&rpc, vec![add_wallet], &signers.rabovel_admin, None).await?,
                );
            }

            let token_account = get_associated_token_address_with_program_id(
                &settlement_wallet,
                &mint,
                &spl_token_2022::id(),
            );
            let token_account_exists = rpc
                .get_account_with_commitment(&token_account, CommitmentConfig::confirmed())
                .await
                .map_err(|error| format!("could not inspect settlement token account: {error}"))?
                .value
                .is_some();
            if !token_account_exists {
                let create_account = create_associated_token_account_idempotent(
                    &admin,
                    &settlement_wallet,
                    &mint,
                    &spl_token_2022::id(),
                );
                signatures.push(
                    Self::submit(&rpc, vec![create_account], &signers.rabovel_admin, None).await?,
                );
            }

            let mint_config = TokenAclBuilder::mint_config_address(&mint);
            let rpc_ref = &rpc;
            let fetch = move |address: Pubkey| async move {
                rpc_ref
                    .get_account_with_commitment(&address, CommitmentConfig::confirmed())
                    .await
                    .map(|response| response.value.map(|account| account.data))
                    .map_err(Into::into)
            };
            let thaw = token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
                &admin,
                &token_account,
                &mint,
                &mint_config,
                &spl_token_2022::id(),
                &settlement_wallet,
                true,
                fetch,
            )
            .await
            .map_err(|error| format!("could not prepare settlement account thaw: {error}"))?;
            signatures.push(Self::submit(&rpc, vec![thaw], &signers.rabovel_admin, None).await?);
            if !Self::token_account_is_initialized(&rpc, token_account, mint, settlement_wallet)
                .await?
            {
                return Err("settlement token account remained frozen after ACL thaw".into());
            }

            let mint_to = spl_token_2022::instruction::mint_to_checked(
                &spl_token_2022::id(),
                &mint,
                &token_account,
                &signers.issuer.pubkey(),
                &[],
                asset.draft.authorized_units,
                0,
            )
            .map_err(|error| format!("could not prepare initial mint: {error}"))?;
            signatures.push(
                Self::submit(
                    &rpc,
                    vec![mint_to],
                    &signers.rabovel_admin,
                    Some(&signers.issuer),
                )
                .await?,
            );
            self.inventory_snapshot(&rpc, asset, operation, admin, settlement_wallet, signatures)
                .await
        }
        .await;
        run.map_err(|message: String| SetupExecutionError {
            reconciliation_required: message.contains("not confirmed"),
            message,
        })
    }

    async fn investor_position(
        &self,
        operation: &AssetSetupOperation,
        wallet: &str,
    ) -> Result<InvestorAssetPosition, SetupExecutionError> {
        let run = async {
            if operation.status != "confirmed" {
                return Err("asset mint setup is not confirmed".into());
            }
            let mint = operation
                .mint_address
                .parse::<Pubkey>()
                .map_err(|_| "stored mint address is invalid".to_string())?;
            let wallet = wallet
                .parse::<Pubkey>()
                .map_err(|_| "linked Solana wallet is invalid".to_string())?;
            let token_account =
                get_associated_token_address_with_program_id(&wallet, &mint, &spl_token_2022::id());
            let rpc =
                RpcClient::new_with_commitment(self.rpc_url.clone(), CommitmentConfig::confirmed());
            let account_ready =
                Self::token_account_is_initialized(&rpc, token_account, mint, wallet).await?;
            let balance = if account_ready {
                rpc.get_token_account_balance(&token_account)
                    .await
                    .map_err(|error| format!("could not read investor equity balance: {error}"))?
                    .amount
            } else {
                "0".into()
            };
            Ok(InvestorAssetPosition {
                token_account: token_account.to_string(),
                balance,
                account_ready,
            })
        }
        .await;
        run.map_err(|message: String| SetupExecutionError {
            reconciliation_required: false,
            message,
        })
    }

    async fn prepare_purchase(
        &self,
        asset: &AssetDraftRecord,
        operation: &AssetSetupOperation,
        terms: PurchaseTerms<'_>,
    ) -> Result<Vec<u8>, SetupExecutionError> {
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
                Self::submit(&rpc, setup, &signers.rabovel_admin, None).await?;
            }
            if !Self::token_account_is_initialized(&rpc, investor_equity, mint, investor).await? {
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
                Self::submit(&rpc, vec![thaw], &signers.rabovel_admin, None).await?;
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
        run.map_err(|message| SetupExecutionError {
            reconciliation_required: false,
            message,
        })
    }

    async fn submit_purchase(
        &self,
        investor_wallet: &str,
        transaction: &[u8],
    ) -> Result<String, SetupExecutionError> {
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
        run.map_err(|message| SetupExecutionError {
            reconciliation_required: message.contains("not confirmed"),
            message,
        })
    }
}
