use super::*;
use async_trait::async_trait;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::issuance::setup::models::AssetSetupOperation;
use solana_adapter::token_acl::TokenAclBuilder;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::AccountMeta;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_associated_token_account_interface::instruction::create_associated_token_account_idempotent;

impl SolanaMintSetupExecutor {
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
        let token_account_ready = solana_adapter::accounts::token_account_is_initialized(
            rpc,
            token_account,
            mint,
            settlement_wallet,
        )
        .await?;
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
}
#[async_trait]
impl InventoryProvider for SolanaMintSetupExecutor {
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
                    solana_adapter::accounts::submit(
                        &rpc,
                        vec![add_wallet],
                        &signers.rabovel_admin,
                        None,
                    )
                    .await?,
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
                    solana_adapter::accounts::submit(
                        &rpc,
                        vec![create_account],
                        &signers.rabovel_admin,
                        None,
                    )
                    .await?,
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
            signatures.push(
                solana_adapter::accounts::submit(&rpc, vec![thaw], &signers.rabovel_admin, None)
                    .await?,
            );
            if !solana_adapter::accounts::token_account_is_initialized(
                &rpc,
                token_account,
                mint,
                settlement_wallet,
            )
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
                solana_adapter::accounts::submit(
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
}
