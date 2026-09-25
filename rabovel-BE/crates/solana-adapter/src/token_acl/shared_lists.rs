use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::{Pubkey, pubkey};

use crate::IssuanceError;

// ABL Gate c525fa710883a55cedd0cd7bf2a5aef6eac05171: CreateList discriminator,
// mode byte, seed pubkey; accounts: authority, payer, list_config, system_program.
const CREATE_LIST_DISCRIMINATOR: u8 = 1;
const LIST_CONFIG_SEED: &[u8] = b"list_config";
pub const ABL_GATE_PROGRAM_ID: Pubkey = pubkey!("GATEzzqxhJnsWF6vHRsgtixxSB8PaQdcqGEVTEHWiULz");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ListMode {
    Allow = 0,
    Block = 2,
}

/// Save these public identifiers in deployment configuration and reuse them for every equity.
#[derive(Debug, Clone, Copy)]
pub struct SharedListSeeds {
    pub allow: Pubkey,
    pub block: Pubkey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedListAddresses {
    pub allow: Pubkey,
    pub block: Pubkey,
}

pub struct SharedListsPlan {
    pub gate_program: Pubkey,
    pub authority: Pubkey,
    pub seeds: SharedListSeeds,
    pub addresses: SharedListAddresses,
    pub instructions: Vec<Instruction>,
    pub required_signers: Vec<Pubkey>,
}

pub struct SharedListsBuilder {
    rabovel_admin: Pubkey,
}

impl SharedListsBuilder {
    pub fn new(rabovel_admin: Pubkey) -> Result<Self, IssuanceError> {
        require_nonzero(rabovel_admin, "Rabovel Admin")?;
        Ok(Self { rabovel_admin })
    }

    pub fn build(
        &self,
        seeds: SharedListSeeds,
        payer: Pubkey,
    ) -> Result<SharedListsPlan, IssuanceError> {
        require_nonzero(payer, "payer")?;
        require_nonzero(seeds.allow, "allow-list seed")?;
        require_nonzero(seeds.block, "block-list seed")?;
        if seeds.allow == seeds.block {
            return Err(IssuanceError::InvalidConfiguration(
                "allow and block lists require different seeds".into(),
            ));
        }

        let allow_address = self.list_address(seeds.allow);
        let block_address = self.list_address(seeds.block);
        let create_allow_list =
            self.create_list_instruction(payer, allow_address, seeds.allow, ListMode::Allow);
        let create_block_list =
            self.create_list_instruction(payer, block_address, seeds.block, ListMode::Block);

        let mut required_signers = vec![payer, self.rabovel_admin];
        required_signers.sort();
        required_signers.dedup();
        Ok(SharedListsPlan {
            gate_program: ABL_GATE_PROGRAM_ID,
            authority: self.rabovel_admin,
            seeds,
            addresses: SharedListAddresses {
                allow: allow_address,
                block: block_address,
            },
            instructions: vec![create_allow_list, create_block_list],
            required_signers,
        })
    }

    fn list_address(&self, seed: Pubkey) -> Pubkey {
        let address_seeds = [LIST_CONFIG_SEED, self.rabovel_admin.as_ref(), seed.as_ref()];
        let (address, _) = Pubkey::find_program_address(&address_seeds, &ABL_GATE_PROGRAM_ID);
        address
    }

    fn create_list_instruction(
        &self,
        payer: Pubkey,
        list_address: Pubkey,
        seed: Pubkey,
        mode: ListMode,
    ) -> Instruction {
        let mut data = Vec::with_capacity(34);
        data.push(CREATE_LIST_DISCRIMINATOR);
        data.push(mode as u8);
        data.extend_from_slice(seed.as_ref());

        let accounts = vec![
            AccountMeta::new_readonly(self.rabovel_admin, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(list_address, false),
            AccountMeta::new_readonly(solana_system_interface::program::id(), false),
        ];
        Instruction {
            program_id: ABL_GATE_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

fn require_nonzero(address: Pubkey, name: &str) -> Result<(), IssuanceError> {
    if address == Pubkey::default() {
        return Err(IssuanceError::InvalidConfiguration(format!(
            "{name} must be nonzero"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_message::Message;

    fn seeds() -> SharedListSeeds {
        SharedListSeeds {
            allow: Pubkey::new_from_array([2; 32]),
            block: Pubkey::new_from_array([3; 32]),
        }
    }

    #[test]
    fn instructions_match_reviewed_gate_create_list_wire_format() {
        let admin = Pubkey::new_from_array([1; 32]);
        let payer = Pubkey::new_from_array([4; 32]);
        let plan = SharedListsBuilder::new(admin)
            .unwrap()
            .build(seeds(), payer)
            .unwrap();
        assert_eq!(plan.instructions.len(), 2);
        assert_ne!(plan.addresses.allow, plan.addresses.block);
        for (instruction, mode, seed, address) in [
            (
                &plan.instructions[0],
                0,
                seeds().allow,
                plan.addresses.allow,
            ),
            (
                &plan.instructions[1],
                2,
                seeds().block,
                plan.addresses.block,
            ),
        ] {
            let mut expected_data = vec![1, mode];
            expected_data.extend_from_slice(seed.as_ref());
            assert_eq!(instruction.data, expected_data);
            assert_eq!(instruction.program_id, ABL_GATE_PROGRAM_ID);
            assert_eq!(
                instruction.accounts,
                vec![
                    AccountMeta::new_readonly(admin, true),
                    AccountMeta::new(payer, true),
                    AccountMeta::new(address, false),
                    AccountMeta::new_readonly(solana_system_interface::program::id(), false),
                ]
            );
            let expected_address = Pubkey::find_program_address(
                &[b"list_config", admin.as_ref(), seed.as_ref()],
                &ABL_GATE_PROGRAM_ID,
            )
            .0;
            assert_eq!(address, expected_address);
        }
        let repeated_plan = SharedListsBuilder::new(admin)
            .unwrap()
            .build(seeds(), payer)
            .unwrap();
        assert_eq!(plan.addresses, repeated_plan.addresses);
    }

    #[test]
    fn signer_plan_matches_message_for_shared_or_separate_fee_payer() {
        let admin = Pubkey::new_unique();
        for payer in [admin, Pubkey::new_unique()] {
            let plan = SharedListsBuilder::new(admin)
                .unwrap()
                .build(seeds(), payer)
                .unwrap();
            let message = Message::new(&plan.instructions, Some(&payer));
            let count = message.header.num_required_signatures as usize;
            let mut signers = message.account_keys[..count].to_vec();
            signers.sort();
            assert_eq!(signers, plan.required_signers);
            assert!(!signers.contains(&plan.addresses.allow));
            assert!(!signers.contains(&plan.addresses.block));
        }
    }

    #[test]
    fn rejects_colliding_seeds_and_empty_configuration() {
        assert!(SharedListsBuilder::new(Pubkey::default()).is_err());
        let admin = Pubkey::new_unique();
        let builder = SharedListsBuilder::new(admin).unwrap();
        assert!(builder.build(seeds(), Pubkey::default()).is_err());
        for invalid_seeds in [
            SharedListSeeds {
                allow: seeds().allow,
                block: seeds().allow,
            },
            SharedListSeeds {
                allow: Pubkey::default(),
                block: seeds().block,
            },
            SharedListSeeds {
                allow: seeds().allow,
                block: Pubkey::default(),
            },
        ] {
            assert!(builder.build(invalid_seeds, admin).is_err());
        }
    }
}
