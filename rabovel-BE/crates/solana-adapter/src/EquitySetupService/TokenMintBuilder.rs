use crate::IssuanceError;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use spl_token_2022::{
    extension::{ExtensionType, default_account_state, metadata_pointer, pausable, transfer_hook},
    instruction::initialize_mint2,
    state::{AccountState, Mint},
};
use spl_token_metadata_interface::{
    instruction as metadata_instruction,
    state::{Field, TokenMetadata as OnChainMetadata},
};

pub(super) fn instruction_error(error: impl std::fmt::Display) -> IssuanceError {
    IssuanceError::Instruction(error.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    RabovelAdmin,
    Issuer,
    TokenAcl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Authority {
    pub role: Role,
    pub address: Pubkey,
}

#[derive(Debug, Clone)]
pub struct MintAuthorities {
    pub mint: Authority,
    pub initial_freeze: Authority,
    pub pause: Authority,
    pub metadata_update: Authority,
    pub metadata_pointer_update: Authority,
    pub transfer_hook_update: Authority,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    #[serde(default)]
    pub additional_metadata: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct TransferHookConfig {
    pub program_id: Option<Pubkey>,
}

#[derive(Debug, Clone)]
pub struct TokenMint {
    pub address: Pubkey,
    pub authorities: MintAuthorities,
    pub metadata: TokenMetadata,
    pub transfer_hook: TransferHookConfig,
}

impl TokenMint {
    pub const DECIMALS: u8 = 0;

    pub(super) fn on_chain_metadata(&self) -> Result<OnChainMetadata, IssuanceError> {
        Ok(OnChainMetadata {
            update_authority: Some(self.authorities.metadata_update.address)
                .try_into()
                .map_err(instruction_error)?,
            mint: self.address,
            name: self.metadata.name.clone(),
            symbol: self.metadata.symbol.clone(),
            uri: self.metadata.uri.clone(),
            additional_metadata: self.metadata.additional_metadata.clone(),
        })
    }
}

pub struct MintCreationPlan {
    pub mint: Pubkey,
    pub instructions: Vec<Instruction>,
    pub required_signers: Vec<Pubkey>,
}

pub struct TokenMintBuilder {
    rabovel_admin: Pubkey,
    issuer: Pubkey,
}

impl TokenMintBuilder {
    /// Addresses must come from trusted service configuration, not the mint request.
    pub fn new(rabovel_admin: Pubkey, issuer: Pubkey) -> Result<Self, IssuanceError> {
        if rabovel_admin == Pubkey::default() || issuer == Pubkey::default() {
            return Err(IssuanceError::InvalidConfiguration(
                "configured admin and issuer addresses must be nonzero".into(),
            ));
        }
        Ok(Self {
            rabovel_admin,
            issuer,
        })
    }

    fn validate_authorities(&self, authorities: &MintAuthorities) -> Result<(), IssuanceError> {
        let expected_authorities = [
            ("mint", &authorities.mint, Role::Issuer, self.issuer),
            (
                "initial_freeze",
                &authorities.initial_freeze,
                Role::RabovelAdmin,
                self.rabovel_admin,
            ),
            (
                "pause",
                &authorities.pause,
                Role::RabovelAdmin,
                self.rabovel_admin,
            ),
            (
                "metadata_update",
                &authorities.metadata_update,
                Role::Issuer,
                self.issuer,
            ),
            (
                "metadata_pointer_update",
                &authorities.metadata_pointer_update,
                Role::Issuer,
                self.issuer,
            ),
            (
                "transfer_hook_update",
                &authorities.transfer_hook_update,
                Role::RabovelAdmin,
                self.rabovel_admin,
            ),
        ];

        for (name, authority, expected_role, expected_address) in expected_authorities {
            if authority.role != expected_role || authority.address != expected_address {
                return Err(IssuanceError::InvalidConfiguration(format!(
                    "{name} authority must be {expected_role:?} at {expected_address}; got {:?} at {}",
                    authority.role, authority.address,
                )));
            }
        }
        Ok(())
    }

    fn base_space() -> Result<usize, IssuanceError> {
        ExtensionType::try_calculate_account_len::<Mint>(&[
            ExtensionType::MetadataPointer,
            ExtensionType::DefaultAccountState,
            ExtensionType::Pausable,
            ExtensionType::TransferHook,
        ])
        .map_err(instruction_error)
    }

    pub fn required_space(&self, spec: &TokenMint) -> Result<usize, IssuanceError> {
        self.validate(spec)?;

        let base_bytes = Self::base_space()?;
        let metadata = spec.on_chain_metadata()?;
        let metadata_bytes = metadata.tlv_size_of().map_err(instruction_error)?;
        let total_bytes = base_bytes
            .checked_add(metadata_bytes)
            .ok_or_else(|| IssuanceError::InvalidConfiguration("mint size overflow".into()))?;

        Ok(total_bytes)
    }

    pub fn validate(&self, spec: &TokenMint) -> Result<(), IssuanceError> {
        let fail = |message: &str| IssuanceError::InvalidConfiguration(message.into());
        if spec.address == Pubkey::default() {
            return Err(fail("mint address must be nonzero"));
        }
        self.validate_authorities(&spec.authorities)?;
        if spec.metadata.name.trim().is_empty()
            || spec.metadata.symbol.trim().is_empty()
            || spec.metadata.uri.trim().is_empty()
        {
            return Err(fail("metadata name, symbol and URI are required"));
        }
        let mut keys = std::collections::HashSet::new();
        for (key, _) in &spec.metadata.additional_metadata {
            if key.trim().is_empty() || !keys.insert(key) {
                return Err(fail("additional metadata keys must be nonempty and unique"));
            }
        }
        Ok(())
    }

    /// Fund rent for required_space; metadata instructions grow the base account.
    pub fn build(
        &self,
        spec: &TokenMint,
        payer: Pubkey,
        rent_lamports: u64,
    ) -> Result<MintCreationPlan, IssuanceError> {
        self.validate(spec)?;
        let instructions = Self::build_instructions(spec, payer, rent_lamports)?;
        let mint = spec.address;
        let authorities = &spec.authorities;
        let mut required_signers = vec![payer, mint, authorities.mint.address];
        if !spec.metadata.additional_metadata.is_empty() {
            required_signers.push(authorities.metadata_update.address);
        }
        required_signers.sort();
        required_signers.dedup();
        Ok(MintCreationPlan {
            mint,
            instructions,
            required_signers,
        })
    }

    fn build_instructions(
        spec: &TokenMint,
        payer: Pubkey,
        rent_lamports: u64,
    ) -> Result<Vec<Instruction>, IssuanceError> {
        let token_program = spl_token_2022::id();
        let mint = spec.address;
        let authorities = &spec.authorities;
        let account_space_bytes = Self::base_space()? as u64;

        let create_mint_account = solana_system_interface::instruction::create_account(
            &payer,
            &mint,
            rent_lamports,
            account_space_bytes,
            &token_program,
        );

        let initialize_metadata_pointer = metadata_pointer::instruction::initialize(
            &token_program,
            &mint,
            Some(authorities.metadata_pointer_update.address),
            Some(mint),
        )
        .map_err(instruction_error)?;

        let initialize_frozen_default =
            default_account_state::instruction::initialize_default_account_state(
                &token_program,
                &mint,
                &AccountState::Frozen,
            )
            .map_err(instruction_error)?;

        let initialize_pause_control =
            pausable::instruction::initialize(&token_program, &mint, &authorities.pause.address)
                .map_err(instruction_error)?;

        let initialize_transfer_hook = transfer_hook::instruction::initialize(
            &token_program,
            &mint,
            Some(authorities.transfer_hook_update.address),
            spec.transfer_hook.program_id,
        )
        .map_err(instruction_error)?;

        let initialize_mint = initialize_mint2(
            &token_program,
            &mint,
            &authorities.mint.address,
            Some(&authorities.initial_freeze.address),
            TokenMint::DECIMALS,
        )
        .map_err(instruction_error)?;

        let initialize_token_metadata = metadata_instruction::initialize(
            &token_program,
            &mint,
            &authorities.metadata_update.address,
            &mint,
            &authorities.mint.address,
            spec.metadata.name.clone(),
            spec.metadata.symbol.clone(),
            spec.metadata.uri.clone(),
        );

        let mut instructions = vec![
            create_mint_account,
            initialize_metadata_pointer,
            initialize_frozen_default,
            initialize_pause_control,
            initialize_transfer_hook,
            initialize_mint,
            initialize_token_metadata,
        ];

        for (key, value) in &spec.metadata.additional_metadata {
            let update_metadata_field = metadata_instruction::update_field(
                &token_program,
                &mint,
                &authorities.metadata_update.address,
                Field::Key(key.clone()),
                value.clone(),
            );
            instructions.push(update_metadata_field);
        }

        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_message::Message;
    use spl_token_2022::instruction::TokenInstruction;

    fn admin_address() -> Pubkey {
        Pubkey::new_from_array([1; 32])
    }

    fn issuer_address() -> Pubkey {
        Pubkey::new_from_array([2; 32])
    }

    fn builder() -> TokenMintBuilder {
        TokenMintBuilder::new(admin_address(), issuer_address()).unwrap()
    }

    fn spec() -> TokenMint {
        let admin = Authority {
            role: Role::RabovelAdmin,
            address: admin_address(),
        };
        let issuer = Authority {
            role: Role::Issuer,
            address: issuer_address(),
        };
        TokenMint {
            address: Pubkey::new_unique(),
            authorities: MintAuthorities {
                mint: issuer,
                initial_freeze: admin,
                pause: admin,
                metadata_update: issuer,
                metadata_pointer_update: issuer,
                transfer_hook_update: admin,
            },
            metadata: TokenMetadata {
                name: "Demo equity".into(),
                symbol: "DEMO".into(),
                uri: "ipfs://demo".into(),
                additional_metadata: vec![],
            },
            transfer_hook: TransferHookConfig::default(),
        }
    }

    #[test]
    fn authority_role_and_address_are_checked_for_every_permission() {
        for index in 0..6 {
            for wrong_role in [false, true] {
                let mut spec = spec();
                let authorities = &mut spec.authorities;
                let authority = match index {
                    0 => &mut authorities.mint,
                    1 => &mut authorities.initial_freeze,
                    2 => &mut authorities.pause,
                    3 => &mut authorities.metadata_update,
                    4 => &mut authorities.metadata_pointer_update,
                    _ => &mut authorities.transfer_hook_update,
                };
                if wrong_role {
                    authority.role = Role::TokenAcl;
                } else {
                    authority.address = Pubkey::new_unique();
                }
                assert!(builder().build(&spec, admin_address(), 1_000_000).is_err());
                assert!(builder().required_space(&spec).is_err());
            }
        }
    }

    #[test]
    fn trusted_configuration_rejects_zero_addresses() {
        assert!(TokenMintBuilder::new(Pubkey::default(), issuer_address()).is_err());
        assert!(TokenMintBuilder::new(admin_address(), Pubkey::default()).is_err());
    }

    #[test]
    fn admin_and_issuer_may_share_a_key_but_keep_their_roles() {
        let mut spec = spec();
        spec.authorities.mint.address = admin_address();
        spec.authorities.metadata_update.address = admin_address();
        spec.authorities.metadata_pointer_update.address = admin_address();
        let builder = TokenMintBuilder::new(admin_address(), admin_address()).unwrap();
        assert!(builder.build(&spec, admin_address(), 1_000_000).is_ok());
    }

    #[test]
    fn creation_has_no_issuance_or_permanent_delegate() {
        let spec = spec();
        let plan = builder()
            .build(&spec, Pubkey::new_unique(), 1_000_000)
            .unwrap();
        let core: Vec<_> = plan
            .instructions
            .iter()
            .skip(1)
            .take(5)
            .map(|ix| TokenInstruction::unpack(&ix.data).unwrap())
            .collect();
        assert!(core.iter().all(|ix| !matches!(
            ix,
            TokenInstruction::InitializePermanentDelegate { .. }
                | TokenInstruction::MintTo { .. }
                | TokenInstruction::MintToChecked { .. }
        )));
        assert!(matches!(
            core.last().unwrap(),
            TokenInstruction::InitializeMint2 { decimals: 0, .. }
        ));
        let hook = &plan.instructions[4].data;
        assert!(hook[hook.len() - 32..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn signer_plan_matches_transaction_including_metadata_updates() {
        let mut spec = spec();
        let payer = Pubkey::new_unique();
        for fields in [vec![], vec![("ratio".into(), "1:1".into())]] {
            spec.metadata.additional_metadata = fields;
            let plan = builder().build(&spec, payer, 1_000_000).unwrap();
            let message = Message::new(&plan.instructions, Some(&payer));
            let mut actual =
                message.account_keys[..message.header.num_required_signatures as usize].to_vec();
            actual.sort();
            assert_eq!(actual, plan.required_signers);
        }
    }

    #[test]
    fn metadata_space_grows_and_ambiguous_keys_are_rejected() {
        let mut spec = spec();
        let initial = builder().required_space(&spec).unwrap();
        spec.metadata
            .additional_metadata
            .push(("ratio".into(), "1:1".into()));
        assert!(builder().required_space(&spec).unwrap() > initial);
        spec.metadata
            .additional_metadata
            .push(("ratio".into(), "2:1".into()));
        assert!(builder().required_space(&spec).is_err());
    }

    #[test]
    fn mint_and_extensions_preserve_configured_addresses() {
        use spl_token_2022::instruction::decode_instruction_data;

        let spec = spec();
        let plan = builder()
            .build(&spec, Pubkey::new_unique(), 1_000_000)
            .unwrap();
        assert_eq!(plan.mint, spec.address);
        for instruction in plan.instructions.iter().skip(1) {
            assert_eq!(instruction.program_id, spl_token_2022::id());
            assert_eq!(instruction.accounts[0].pubkey, spec.address);
        }

        let pointer_data = &plan.instructions[1].data[1..];
        let pointer = decode_instruction_data::<
            metadata_pointer::instruction::InitializeInstructionData,
        >(pointer_data)
        .unwrap();
        assert_eq!(
            Option::<Pubkey>::from(pointer.authority),
            Some(spec.authorities.metadata_pointer_update.address)
        );
        assert_eq!(
            Option::<Pubkey>::from(pointer.metadata_address),
            Some(spec.address)
        );

        let frozen_data = &plan.instructions[2].data;
        assert_eq!(frozen_data.last(), Some(&(AccountState::Frozen as u8)));

        let pause_data = &plan.instructions[3].data[1..];
        let pause =
            decode_instruction_data::<pausable::instruction::InitializeInstructionData>(pause_data)
                .unwrap();
        assert_eq!(pause.authority, spec.authorities.pause.address);

        let hook_data = &plan.instructions[4].data[1..];
        let hook =
            decode_instruction_data::<transfer_hook::instruction::InitializeInstructionData>(
                hook_data,
            )
            .unwrap();
        assert_eq!(
            Option::<Pubkey>::from(hook.authority),
            Some(spec.authorities.transfer_hook_update.address)
        );
        assert_eq!(Option::<Pubkey>::from(hook.program_id), None);

        let mint_instruction = TokenInstruction::unpack(&plan.instructions[5].data).unwrap();
        match mint_instruction {
            TokenInstruction::InitializeMint2 {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                assert_eq!(decimals, 0);
                assert_eq!(mint_authority, spec.authorities.mint.address);
                assert_eq!(
                    freeze_authority,
                    Some(spec.authorities.initial_freeze.address).into()
                );
            }
            _ => panic!("expected mint initialization after extension setup"),
        }
    }

    #[test]
    fn metadata_instructions_preserve_values_and_update_authority() {
        use spl_token_metadata_interface::instruction::TokenMetadataInstruction;

        let mut spec = spec();
        spec.metadata
            .additional_metadata
            .push(("underlying".into(), "NGX:DANGCEM".into()));
        let plan = builder()
            .build(&spec, Pubkey::new_unique(), 1_000_000)
            .unwrap();
        let metadata_instruction = &plan.instructions[6];
        assert_eq!(
            metadata_instruction.accounts[1].pubkey,
            spec.authorities.metadata_update.address
        );
        assert_eq!(metadata_instruction.accounts[2].pubkey, spec.address);
        assert_eq!(
            metadata_instruction.accounts[3].pubkey,
            spec.authorities.mint.address
        );
        match TokenMetadataInstruction::unpack(&metadata_instruction.data).unwrap() {
            TokenMetadataInstruction::Initialize(metadata) => {
                assert_eq!(metadata.name, spec.metadata.name);
                assert_eq!(metadata.symbol, spec.metadata.symbol);
                assert_eq!(metadata.uri, spec.metadata.uri);
            }
            _ => panic!("expected metadata initialization"),
        }
        match TokenMetadataInstruction::unpack(&plan.instructions[7].data).unwrap() {
            TokenMetadataInstruction::UpdateField(update) => {
                assert_eq!(update.field, Field::Key("underlying".into()));
                assert_eq!(update.value, "NGX:DANGCEM");
            }
            _ => panic!("expected additional metadata update"),
        }
    }

    #[test]
    fn invalid_inputs_are_rejected_by_build() {
        for index in 0..11 {
            let mut spec = spec();
            match index {
                0 => spec.address = Pubkey::default(),
                1 => spec.authorities.mint.address = Pubkey::default(),
                2 => spec.authorities.initial_freeze.address = Pubkey::default(),
                3 => spec.authorities.pause.address = Pubkey::default(),
                4 => spec.authorities.metadata_update.address = Pubkey::default(),
                5 => spec.authorities.metadata_pointer_update.address = Pubkey::default(),
                6 => spec.authorities.transfer_hook_update.address = Pubkey::default(),
                7 => spec.metadata.name = " ".into(),
                8 => spec.metadata.symbol.clear(),
                9 => spec.metadata.uri.clear(),
                _ => spec
                    .metadata
                    .additional_metadata
                    .push((" ".into(), "value".into())),
            }
            let result = builder().build(&spec, Pubkey::new_unique(), 1_000_000);
            assert!(
                matches!(result, Err(IssuanceError::InvalidConfiguration(_))),
                "case {index}"
            );
        }
    }

    #[test]
    fn shared_authority_keys_only_require_one_signature_per_key() {
        let mut spec = spec();
        let issuer = spec.authorities.mint.address;
        spec.authorities.metadata_update.address = issuer;
        spec.metadata
            .additional_metadata
            .push(("ratio".into(), "1:1".into()));
        let plan = builder().build(&spec, issuer, 1_000_000).unwrap();
        assert_eq!(plan.required_signers.len(), 2);
        assert!(plan.required_signers.contains(&issuer));
        assert!(plan.required_signers.contains(&spec.address));
    }
}
