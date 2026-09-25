use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use spl_token_2022::{
    extension::{
        BaseStateWithExtensions, StateWithExtensions, default_account_state::DefaultAccountState,
    },
    state::{AccountState, Mint},
};
use token_acl_client::{
    accounts::{MINT_CONFIG_DISCRIMINATOR, MintConfig},
    instructions::{CreateConfig, CreateConfigInstructionArgs},
    programs::TOKEN_ACL_ID,
};

use crate::IssuanceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AclConfigState {
    AwaitingGateSetup,
    PermissionlessThawEnabled,
}

pub struct AclSetupPlan {
    pub mint: Pubkey,
    pub mint_config: Pubkey,
    pub instructions: Vec<Instruction>,
    pub required_signers: Vec<Pubkey>,
}

pub struct TokenAclBuilder {
    rabovel_admin: Pubkey,
    gate_program: Pubkey,
}

impl TokenAclBuilder {
    /// Both addresses come from trusted deployment configuration.
    pub fn new(rabovel_admin: Pubkey, gate_program: Pubkey) -> Result<Self, IssuanceError> {
        if rabovel_admin == Pubkey::default() || gate_program == Pubkey::default() {
            return Err(invalid("admin and Gate program addresses must be nonzero"));
        }
        Ok(Self {
            rabovel_admin,
            gate_program,
        })
    }

    pub fn program_id() -> Pubkey {
        TOKEN_ACL_ID
    }

    pub fn mint_config_address(mint: &Pubkey) -> Pubkey {
        MintConfig::find_pda(mint).0
    }

    pub fn build_handover(
        &self,
        mint_address: Pubkey,
        mint_owner: Pubkey,
        mint_data: &[u8],
        payer: Pubkey,
    ) -> Result<AclSetupPlan, IssuanceError> {
        if mint_address == Pubkey::default() || payer == Pubkey::default() {
            return Err(invalid("mint and payer addresses must be nonzero"));
        }
        self.validate_mint(mint_owner, mint_data, self.rabovel_admin)?;
        let mint_config = Self::mint_config_address(&mint_address);
        let create_config = CreateConfig {
            payer,
            authority: self.rabovel_admin,
            mint: mint_address,
            mint_config,
            system_program: solana_system_interface::program::id(),
            token_program: spl_token_2022::id(),
        };
        let arguments = CreateConfigInstructionArgs {
            gating_program: self.gate_program,
        };
        let create_config_instruction = create_config.instruction(arguments);
        let mut required_signers = vec![payer, self.rabovel_admin];
        required_signers.sort();
        required_signers.dedup();
        Ok(AclSetupPlan {
            mint: mint_address,
            mint_config,
            instructions: vec![create_config_instruction],
            required_signers,
        })
    }

    pub fn verify_handover(
        &self,
        mint_address: Pubkey,
        mint_owner: Pubkey,
        mint_data: &[u8],
        config_address: Pubkey,
        config_owner: Pubkey,
        config_data: &[u8],
    ) -> Result<AclConfigState, IssuanceError> {
        let (expected_address, expected_bump) = MintConfig::find_pda(&mint_address);
        if config_address != expected_address || config_owner != TOKEN_ACL_ID {
            return Err(invalid("incorrect MintConfig address or program owner"));
        }
        // The generated client's LEN is zero in 0.3.1; the program stores four bytes and three keys.
        let expected_config_bytes = 4 + 3 * 32;
        if config_data.len() != expected_config_bytes {
            return Err(invalid("incorrect MintConfig data length"));
        }
        let config = MintConfig::from_bytes(config_data)
            .map_err(|error| invalid(&format!("invalid MintConfig data: {error}")))?;
        if config.discriminator != MINT_CONFIG_DISCRIMINATOR
            || config.bump != expected_bump
            || config.mint != mint_address
            || config.freeze_authority != self.rabovel_admin
            || config.gating_program != self.gate_program
            || config.enable_permissionless_freeze
        {
            return Err(invalid(
                "MintConfig does not match Rabovel authority and Gate policy",
            ));
        }
        self.validate_mint(mint_owner, mint_data, expected_address)?;
        let state = if config.enable_permissionless_thaw {
            AclConfigState::PermissionlessThawEnabled
        } else {
            AclConfigState::AwaitingGateSetup
        };
        Ok(state)
    }

    fn validate_mint(
        &self,
        owner: Pubkey,
        data: &[u8],
        expected_freeze_authority: Pubkey,
    ) -> Result<(), IssuanceError> {
        if owner != spl_token_2022::id() {
            return Err(invalid("equity mint must be owned by Token-2022"));
        }
        let mint = StateWithExtensions::<Mint>::unpack(data)
            .map_err(|error| invalid(&format!("invalid mint data: {error}")))?;
        if mint.base.freeze_authority != Some(expected_freeze_authority).into() {
            return Err(invalid("mint has an unexpected freeze authority"));
        }
        let default_state = mint
            .get_extension::<DefaultAccountState>()
            .map_err(|_| invalid("mint requires DefaultAccountState"))?;
        if default_state.state != AccountState::Frozen as u8 {
            return Err(invalid("new equity token accounts must default to frozen"));
        }
        Ok(())
    }
}

fn invalid(message: &str) -> IssuanceError {
    IssuanceError::InvalidConfiguration(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_message::Message;
    use spl_token_2022::extension::{
        BaseStateWithExtensionsMut, ExtensionType, StateWithExtensionsMut,
    };

    fn mint_data(freeze_authority: Pubkey, frozen: bool) -> Vec<u8> {
        let size =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::DefaultAccountState])
                .unwrap();
        let mut data = vec![0; size];
        let mut mint = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut data).unwrap();
        let default_state = mint.init_extension::<DefaultAccountState>(true).unwrap();
        default_state.state = if frozen {
            AccountState::Frozen
        } else {
            AccountState::Initialized
        } as u8;
        mint.base = Mint {
            mint_authority: Some(Pubkey::new_unique()).into(),
            supply: 0,
            decimals: 0,
            is_initialized: true,
            freeze_authority: Some(freeze_authority).into(),
        };
        mint.pack_base();
        mint.init_account_type().unwrap();
        data
    }

    fn config_data(mint: Pubkey, admin: Pubkey, gate: Pubkey) -> Vec<u8> {
        let bump = MintConfig::find_pda(&mint).1;
        let mut data = vec![MINT_CONFIG_DISCRIMINATOR, bump, 0, 0];
        data.extend_from_slice(mint.as_ref());
        data.extend_from_slice(admin.as_ref());
        data.extend_from_slice(gate.as_ref());
        data
    }

    #[test]
    fn handover_uses_admin_signature_and_sdk_pda_without_enabling_thaw() {
        let admin = Pubkey::new_unique();
        let gate = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let builder = TokenAclBuilder::new(admin, gate).unwrap();
        let data = mint_data(admin, true);
        for payer in [admin, Pubkey::new_unique()] {
            let plan = builder
                .build_handover(mint, spl_token_2022::id(), &data, payer)
                .unwrap();
            assert_eq!(plan.mint_config, MintConfig::find_pda(&mint).0);
            assert_eq!(plan.instructions.len(), 1);
            let instruction = &plan.instructions[0];
            assert_eq!(instruction.program_id, TOKEN_ACL_ID);
            assert_eq!(instruction.data[0], 0);
            assert_eq!(&instruction.data[1..], gate.as_ref());
            assert_eq!(instruction.accounts[1].pubkey, admin);
            assert!(instruction.accounts[1].is_signer);
            let message = Message::new(&plan.instructions, Some(&payer));
            let signer_count = message.header.num_required_signatures as usize;
            let mut signers = message.account_keys[..signer_count].to_vec();
            signers.sort();
            assert_eq!(signers, plan.required_signers);
        }
    }

    #[test]
    fn handover_rejects_wrong_mint_owner_authority_or_default_state() {
        let admin = Pubkey::new_unique();
        let builder = TokenAclBuilder::new(admin, Pubkey::new_unique()).unwrap();
        let mint = Pubkey::new_unique();
        let valid_data = mint_data(admin, true);
        let wrong_authority = mint_data(Pubkey::new_unique(), true);
        let unfrozen = mint_data(admin, false);
        for (owner, data) in [
            (Pubkey::new_unique(), valid_data.as_slice()),
            (spl_token_2022::id(), wrong_authority.as_slice()),
            (spl_token_2022::id(), unfrozen.as_slice()),
            (spl_token_2022::id(), &[]),
        ] {
            assert!(builder.build_handover(mint, owner, data, admin).is_err());
        }
    }

    #[test]
    fn readback_validates_config_identity_and_freeze_delegation() {
        let admin = Pubkey::new_unique();
        let gate = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let config = TokenAclBuilder::mint_config_address(&mint);
        let builder = TokenAclBuilder::new(admin, gate).unwrap();
        let delegated_mint = mint_data(config, true);
        let data = config_data(mint, admin, gate);
        let verify = |address, owner, bytes: &[u8]| {
            builder.verify_handover(
                mint,
                spl_token_2022::id(),
                &delegated_mint,
                address,
                owner,
                bytes,
            )
        };
        assert_eq!(
            verify(config, TOKEN_ACL_ID, &data).unwrap(),
            AclConfigState::AwaitingGateSetup
        );
        let mut enabled_data = data.clone();
        enabled_data[2] = 1;
        assert_eq!(
            verify(config, TOKEN_ACL_ID, &enabled_data).unwrap(),
            AclConfigState::PermissionlessThawEnabled
        );
        assert!(verify(Pubkey::new_unique(), TOKEN_ACL_ID, &data).is_err());
        assert!(verify(config, Pubkey::new_unique(), &data).is_err());
        for offset in [0, 1, 3, 4, 36, 68] {
            let mut invalid_data = data.clone();
            invalid_data[offset] ^= 1;
            assert!(
                verify(config, TOKEN_ACL_ID, &invalid_data).is_err(),
                "offset {offset}"
            );
        }
        assert!(verify(config, TOKEN_ACL_ID, &data[..99]).is_err());
        let undelegated_mint = mint_data(admin, true);
        assert!(
            builder
                .verify_handover(
                    mint,
                    spl_token_2022::id(),
                    &undelegated_mint,
                    config,
                    TOKEN_ACL_ID,
                    &data
                )
                .is_err()
        );
    }

    #[test]
    fn deployment_configuration_rejects_empty_identities() {
        assert!(TokenAclBuilder::new(Pubkey::default(), Pubkey::new_unique()).is_err());
        assert!(TokenAclBuilder::new(Pubkey::new_unique(), Pubkey::default()).is_err());
    }
}
