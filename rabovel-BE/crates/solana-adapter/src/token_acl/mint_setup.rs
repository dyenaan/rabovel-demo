//! Per-mint ACL setup against the reviewed ABL Gate c525fa7 interface.
//! Each confirmed mutation is read back before the next permission is granted.
use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use token_acl_client::instructions::{
    TogglePermissionlessInstructions, TogglePermissionlessInstructionsInstructionArgs,
};
use token_acl_interface::{
    THAW_EXTRA_ACCOUNT_METAS_SEED, instruction::CanThawPermissionlessInstruction,
};

use super::{
    ABL_GATE_PROGRAM_ID, AclConfigState, AclDeploymentConfig, ListMode, SharedListsPlan,
    TokenAclBuilder, list_setup::verify_list,
};
use crate::IssuanceError;

/// An account read at the requested address by the transport.
#[derive(Clone, Debug)]
pub struct AclAccount {
    pub owner: Pubkey,
    pub data: Vec<u8>,
    pub executable: bool,
}

/// Injectable chain boundary. Implementations must return accounts in requested order,
/// and confirm at the same commitment used for subsequent reads.
#[async_trait]
pub trait MintAclTransport: Send + Sync {
    async fn accounts(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Option<AclAccount>>, IssuanceError>;

    /// An uncertain submission must return an error containing the signed transaction's
    /// signature. Callers must reconcile it; this service never retries a failed send.
    async fn submit_and_confirm(
        &self,
        instruction: Instruction,
        admin: &Keypair,
    ) -> Result<String, IssuanceError>;
}

#[async_trait]
impl MintAclTransport for RpcClient {
    async fn accounts(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Option<AclAccount>>, IssuanceError> {
        if !self.commitment().is_at_least_confirmed() {
            return Err(invalid(
                "ACL setup requires confirmed or finalized commitment",
            ));
        }
        Ok(self
            .get_multiple_accounts(addresses)
            .await
            .map_err(rpc_error)?
            .into_iter()
            .map(|account| {
                account.map(|account| AclAccount {
                    owner: account.owner,
                    data: account.data,
                    executable: account.executable,
                })
            })
            .collect())
    }

    async fn submit_and_confirm(
        &self,
        instruction: Instruction,
        admin: &Keypair,
    ) -> Result<String, IssuanceError> {
        if !self.commitment().is_at_least_confirmed() {
            return Err(invalid(
                "ACL setup requires confirmed or finalized commitment",
            ));
        }
        let blockhash = self.get_latest_blockhash().await.map_err(rpc_error)?;
        let mut transaction =
            Transaction::new_unsigned(Message::new(&[instruction], Some(&admin.pubkey())));
        transaction
            .try_sign(&[admin], blockhash)
            .map_err(|error| IssuanceError::Transaction(error.to_string()))?;
        let signature = transaction.signatures[0].to_string();
        self.send_and_confirm_transaction(&transaction).await.map_err(|error| rpc_error(format!(
            "ACL setup transaction {signature}: {error}; reconcile the signature and account state before retrying"
        )))?;
        Ok(signature)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Handover,
    AssociateLists,
    EnableThaw,
    Ready,
}

#[derive(Debug)]
pub struct MintAclResult {
    pub mint: Pubkey,
    pub mint_config: Pubkey,
    pub thaw_extra_metas: Pubkey,
    /// Only transactions submitted by this invocation; empty for an already-ready mint.
    pub signatures: Vec<String>,
}

pub struct MintAclService<T> {
    transport: T,
}

impl<T: MintAclTransport> MintAclService<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Configure one existing equity mint using trusted saved deployment configuration.
    /// Shared lists must already exist. Conflicting state is rejected, never overwritten.
    /// This is not a durable worker: persist signed operations before wiring to event replay.
    pub async fn configure(
        &self,
        config: &AclDeploymentConfig,
        mint: Pubkey,
        admin: &Keypair,
    ) -> Result<MintAclResult, IssuanceError> {
        let lists = config.plan(admin.pubkey())?;
        let mut result = MintAclResult {
            mint,
            mint_config: TokenAclBuilder::mint_config_address(&mint),
            thaw_extra_metas: thaw_extra_metas_address(&mint),
            signatures: Vec::new(),
        };
        self.configure_inner(mint, admin, &lists, &mut result.signatures).await.map_err(|error| {
            if result.signatures.is_empty() { error } else {
                IssuanceError::Transaction(format!(
                    "ACL setup stopped: {error}; previously confirmed signatures: {}; inspect state before retrying",
                    result.signatures.join(", ")
                ))
            }
        })?;
        Ok(result)
    }

    async fn configure_inner(
        &self,
        mint: Pubkey,
        admin: &Keypair,
        lists: &SharedListsPlan,
        signatures: &mut Vec<String>,
    ) -> Result<(), IssuanceError> {
        if mint == Pubkey::default() {
            return Err(invalid("mint must be nonzero"));
        }
        let addresses = [
            mint,
            TokenAclBuilder::mint_config_address(&mint),
            thaw_extra_metas_address(&mint),
            lists.addresses.allow,
            lists.addresses.block,
            TokenAclBuilder::program_id(),
            ABL_GATE_PROGRAM_ID,
        ];
        let mut previous = None;
        // Three mutations at most, plus a final readback. Never resubmit on stale reads.
        for _ in 0..4 {
            let accounts = self.transport.accounts(&addresses).await?;
            let (stage, instruction) = next_step(mint, lists, &accounts)?;
            if previous.is_some_and(|previous| stage <= previous) {
                return Err(invalid(
                    "confirmed ACL setup did not advance; reconcile account state",
                ));
            }
            if stage == Stage::Ready {
                return Ok(());
            }
            let instruction =
                instruction.ok_or_else(|| invalid("missing ACL setup instruction"))?;
            signatures.push(
                self.transport
                    .submit_and_confirm(instruction, admin)
                    .await?,
            );
            previous = Some(stage);
        }
        Err(invalid("ACL setup requires reconciliation"))
    }
}

pub fn thaw_extra_metas_address(mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[THAW_EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()],
        &ABL_GATE_PROGRAM_ID,
    )
    .0
}

fn next_step(
    mint: Pubkey,
    lists: &SharedListsPlan,
    accounts: &[Option<AclAccount>],
) -> Result<(Stage, Option<Instruction>), IssuanceError> {
    if accounts.len() != 7 {
        return Err(invalid("incomplete ACL account snapshot"));
    }
    let required = |index: usize| {
        accounts[index]
            .as_ref()
            .ok_or_else(|| invalid("required ACL setup account is missing"))
    };
    for index in [5, 6] {
        if !required(index)?.executable {
            return Err(invalid("ACL and Gate programs must be executable"));
        }
    }
    for (index, seed, mode) in [
        (3, lists.seeds.allow, ListMode::Allow),
        (4, lists.seeds.block, ListMode::Block),
    ] {
        let account = required(index)?;
        if account.executable {
            return Err(invalid("list must be a data account"));
        }
        verify_list(account.owner, &account.data, lists.authority, seed, mode)?;
    }
    let mint_account = required(0)?;
    if mint_account.executable {
        return Err(invalid("mint must be a data account"));
    }
    let builder = TokenAclBuilder::new(lists.authority, ABL_GATE_PROGRAM_ID)?;
    let Some(config) = &accounts[1] else {
        if accounts[2].is_some() {
            return Err(invalid("Gate metadata exists without MintConfig"));
        }
        let mut plan = builder.build_handover(
            mint,
            mint_account.owner,
            &mint_account.data,
            lists.authority,
        )?;
        return Ok((Stage::Handover, plan.instructions.pop()));
    };
    if config.executable {
        return Err(invalid("MintConfig must be a data account"));
    }
    let state = builder.verify_handover(
        mint,
        mint_account.owner,
        &mint_account.data,
        TokenAclBuilder::mint_config_address(&mint),
        config.owner,
        &config.data,
    )?;
    match &accounts[2] {
        None => {
            if state == AclConfigState::PermissionlessThawEnabled {
                return Err(invalid(
                    "permissionless thaw is enabled without verified Gate metadata",
                ));
            }
            Ok((
                Stage::AssociateLists,
                Some(associate_lists_instruction(mint, lists)),
            ))
        }
        Some(metadata) => {
            if metadata.executable
                || metadata.owner != ABL_GATE_PROGRAM_ID
                || metadata.data != expected_thaw_metadata(lists)?
            {
                return Err(invalid(
                    "Gate thaw metadata does not match the shared allow/block policy",
                ));
            }
            if state == AclConfigState::PermissionlessThawEnabled {
                return Ok((Stage::Ready, None));
            }
            Ok((
                Stage::EnableThaw,
                Some(
                    TogglePermissionlessInstructions {
                        authority: lists.authority,
                        mint_config: TokenAclBuilder::mint_config_address(&mint),
                    }
                    .instruction(
                        TogglePermissionlessInstructionsInstructionArgs {
                            freeze_enabled: false,
                            thaw_enabled: true,
                        },
                    ),
                ),
            ))
        }
    }
}

// Narrow encoder for reviewed Gate SetupExtraMetas (0x04); the published Gate SDK
// has incompatible Solana pins. Account order is defined by that program, not FE input.
fn associate_lists_instruction(mint: Pubkey, lists: &SharedListsPlan) -> Instruction {
    Instruction {
        program_id: ABL_GATE_PROGRAM_ID,
        data: vec![4],
        accounts: vec![
            AccountMeta::new_readonly(lists.authority, true),
            AccountMeta::new(lists.authority, true),
            AccountMeta::new_readonly(TokenAclBuilder::mint_config_address(&mint), false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(thaw_extra_metas_address(&mint), false),
            AccountMeta::new_readonly(solana_system_interface::program::id(), false),
            AccountMeta::new_readonly(lists.addresses.allow, false),
            AccountMeta::new_readonly(lists.addresses.block, false),
        ],
    }
}

fn expected_thaw_metadata(lists: &SharedListsPlan) -> Result<Vec<u8>, IssuanceError> {
    let mut metas = Vec::with_capacity(4);
    for (list, list_index) in [(lists.addresses.allow, 6), (lists.addresses.block, 8)] {
        metas.push(
            ExtraAccountMeta::new_with_pubkey(&list, false, false).map_err(instruction_error)?,
        );
        metas.push(
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: b"wallet_entry".to_vec(),
                    },
                    Seed::AccountKey { index: list_index },
                    // Gate CPI account 1 is the token account; its holder starts at byte 32.
                    // The flag account occupies index 4, followed by metadata at index 5.
                    Seed::AccountData {
                        account_index: 1,
                        data_index: 32,
                        length: 32,
                    },
                ],
                false,
                false,
            )
            .map_err(instruction_error)?,
        );
    }
    let mut data = vec![0; ExtraAccountMetaList::size_of(metas.len()).map_err(instruction_error)?];
    ExtraAccountMetaList::init::<CanThawPermissionlessInstruction>(&mut data, &metas)
        .map_err(instruction_error)?;
    Ok(data)
}

fn invalid(message: &str) -> IssuanceError {
    IssuanceError::InvalidConfiguration(message.into())
}
fn rpc_error(error: impl std::fmt::Display) -> IssuanceError {
    IssuanceError::Rpc(error.to_string())
}
fn instruction_error(error: impl std::fmt::Display) -> IssuanceError {
    IssuanceError::Instruction(error.to_string())
}

#[cfg(test)]
mod tests;
