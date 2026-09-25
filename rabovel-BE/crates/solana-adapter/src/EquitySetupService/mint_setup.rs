use solana_client::nonblocking::rpc_client::RpcClient;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_token_2022::{
    extension::{
        BaseStateWithExtensions, ExtensionType, StateWithExtensions,
        default_account_state::DefaultAccountState, metadata_pointer::MetadataPointer,
        pausable::PausableConfig, transfer_hook::TransferHook,
    },
    state::{AccountState, Mint},
};
use spl_token_metadata_interface::state::TokenMetadata as OnChainMetadata;

use crate::{
    IssuanceError,
    authority_config::{AuthConfig, AuthoritySigners},
    token_acl::{AclSetupPlan, TokenAclBuilder},
    token_mint_builder::{
        Authority, MintAuthorities, Role, TokenMetadata, TokenMint, TransferHookConfig,
    },
};

pub struct EquitySetupService {
    rpc: RpcClient,
}

impl EquitySetupService {
    pub fn new(rpc: RpcClient) -> Self {
        Self { rpc }
    }

    pub async fn prepare_acl_handover(
        &self,
        auth_config: &AuthConfig,
        mint: Pubkey,
        gate_program: Pubkey,
    ) -> Result<AclSetupPlan, IssuanceError> {
        let builder = TokenAclBuilder::new(auth_config.rabovel_admin, gate_program)?;
        for program in [TokenAclBuilder::program_id(), gate_program] {
            let account = self.rpc.get_account(&program).await.map_err(rpc_error)?;
            if !account.executable {
                return Err(IssuanceError::InvalidConfiguration(format!(
                    "required ACL/Gate program {program} is not executable",
                )));
            }
        }
        let config_address = TokenAclBuilder::mint_config_address(&mint);
        let existing_config = self
            .rpc
            .get_account_with_commitment(&config_address, self.rpc.commitment())
            .await
            .map_err(rpc_error)?;
        if existing_config.value.is_some() {
            return Err(IssuanceError::InvalidConfiguration(
                "MintConfig already exists; verify it before continuing".into(),
            ));
        }
        let mint_account = self.rpc.get_account(&mint).await.map_err(rpc_error)?;
        builder.build_handover(
            mint,
            mint_account.owner,
            &mint_account.data,
            auth_config.rabovel_admin,
        )
    }

    pub async fn create_mint(
        &self,
        signers: &AuthoritySigners,
        mint_keypair: &Keypair,
        metadata: TokenMetadata,
    ) -> Result<String, IssuanceError> {
        self.create_or_verify_mint(signers, mint_keypair, metadata)
            .await?
            .ok_or_else(|| {
                IssuanceError::InvalidConfiguration(
                    "mint already exists and matches the requested configuration".into(),
                )
            })
    }

    /// Replay-safe mint creation. If the intended address already exists, verify its full
    /// configuration instead of submitting another transaction.
    pub async fn create_or_verify_mint(
        &self,
        signers: &AuthoritySigners,
        mint_keypair: &Keypair,
        metadata: TokenMetadata,
    ) -> Result<Option<String>, IssuanceError> {
        let auth_config = signers.auth_config();
        let builder = auth_config.mint_builder()?;
        let admin = Authority {
            role: Role::RabovelAdmin,
            address: auth_config.rabovel_admin,
        };
        let issuer = Authority {
            role: Role::Issuer,
            address: auth_config.issuer,
        };
        let spec = TokenMint {
            address: mint_keypair.pubkey(),
            authorities: MintAuthorities {
                mint: issuer,
                initial_freeze: admin,
                pause: admin,
                metadata_update: issuer,
                metadata_pointer_update: issuer,
                transfer_hook_update: admin,
            },
            metadata,
            transfer_hook: TransferHookConfig::default(),
        };
        let required_bytes = builder.required_space(&spec)?;
        let existing_account = self
            .rpc
            .get_account_with_commitment(&spec.address, self.rpc.commitment())
            .await
            .map_err(rpc_error)?;
        if existing_account.value.is_some() {
            self.verify_mint(&spec).await?;
            return Ok(None);
        }
        let rent_lamports = self
            .rpc
            .get_minimum_balance_for_rent_exemption(required_bytes)
            .await
            .map_err(rpc_error)?;
        let payer = admin.address;
        let plan = builder.build(&spec, payer, rent_lamports)?;
        let available_signers = [&signers.rabovel_admin, &signers.issuer, mint_keypair];
        let mut required_signers = Vec::new();
        for address in &plan.required_signers {
            let signer = available_signers
                .iter()
                .find(|signer| signer.pubkey() == *address)
                .ok_or_else(|| {
                    IssuanceError::InvalidConfiguration(format!("missing signer {address}"))
                })?;
            required_signers.push(*signer);
        }
        let blockhash = self.rpc.get_latest_blockhash().await.map_err(rpc_error)?;
        let message = Message::new(&plan.instructions, Some(&payer));
        let mut transaction = Transaction::new_unsigned(message);
        transaction
            .try_sign(&required_signers, blockhash)
            .map_err(|error| IssuanceError::Transaction(error.to_string()))?;
        let signature = transaction.signatures[0].to_string();
        self.rpc.send_and_confirm_transaction(&transaction).await.map_err(|error| {
            IssuanceError::Rpc(format!("mint {} transaction {signature}: {error}; inspect this address before retrying", spec.address))
        })?;
        self.verify_mint(&spec).await.map_err(|error| {
            IssuanceError::Transaction(format!(
                "transaction {signature} confirmed, but mint readback failed: {error}"
            ))
        })?;
        Ok(Some(signature))
    }

    async fn verify_mint(&self, spec: &TokenMint) -> Result<(), IssuanceError> {
        let account = self
            .rpc
            .get_account(&spec.address)
            .await
            .map_err(rpc_error)?;
        let fail =
            || IssuanceError::Transaction("mint does not match requested configuration".into());
        if account.owner != spl_token_2022::id() {
            return Err(fail());
        }
        let mint = StateWithExtensions::<Mint>::unpack(&account.data)
            .map_err(super::token_mint_builder::instruction_error)?;
        let pointer = mint
            .get_extension::<MetadataPointer>()
            .map_err(super::token_mint_builder::instruction_error)?;
        let frozen = mint
            .get_extension::<DefaultAccountState>()
            .map_err(super::token_mint_builder::instruction_error)?;
        let pause = mint
            .get_extension::<PausableConfig>()
            .map_err(super::token_mint_builder::instruction_error)?;
        let hook = mint
            .get_extension::<TransferHook>()
            .map_err(super::token_mint_builder::instruction_error)?;
        let metadata = mint
            .get_variable_len_extension::<OnChainMetadata>()
            .map_err(super::token_mint_builder::instruction_error)?;
        let extensions = mint
            .get_extension_types()
            .map_err(super::token_mint_builder::instruction_error)?;
        let expected_metadata = spec.on_chain_metadata()?;
        let matches = mint.base.is_initialized
            && mint.base.supply == 0
            && mint.base.decimals == TokenMint::DECIMALS
            && mint.base.mint_authority == Some(spec.authorities.mint.address).into()
            && mint.base.freeze_authority == Some(spec.authorities.initial_freeze.address).into()
            && Option::from(pointer.authority)
                == Some(spec.authorities.metadata_pointer_update.address)
            && Option::from(pointer.metadata_address) == Some(spec.address)
            && frozen.state == AccountState::Frozen as u8
            && Option::from(pause.authority) == Some(spec.authorities.pause.address)
            && !bool::from(pause.paused)
            && Option::from(hook.authority) == Some(spec.authorities.transfer_hook_update.address)
            && Option::<Pubkey>::from(hook.program_id).is_none()
            && metadata == expected_metadata
            && !extensions.contains(&ExtensionType::PermanentDelegate);
        if !matches {
            return Err(fail());
        }
        Ok(())
    }
}

fn rpc_error(error: impl std::fmt::Display) -> IssuanceError {
    IssuanceError::Rpc(error.to_string())
}
