use std::{fs::OpenOptions, io::Write, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use super::{ABL_GATE_PROGRAM_ID, SharedListSeeds, SharedListsBuilder, SharedListsPlan};
use crate::IssuanceError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AclDeploymentConfig {
    pub environment: String,
    pub rabovel_admin: String,
    pub gate_program: String,
    pub allow_list_seed: String,
    pub block_list_seed: String,
}

impl AclDeploymentConfig {
    pub fn new_local(admin: Pubkey) -> Self {
        Self {
            environment: "localnet".into(),
            rabovel_admin: admin.to_string(),
            gate_program: ABL_GATE_PROGRAM_ID.to_string(),
            allow_list_seed: Keypair::new().pubkey().to_string(),
            block_list_seed: Keypair::new().pubkey().to_string(),
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        Self::from_json_bytes(&bytes)
    }

    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Never replace saved seeds: they identify the lists across subsequent runs.
    pub fn save_new(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = serde_json::to_vec_pretty(self)?;
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        Ok(())
    }

    pub fn plan(&self, admin: Pubkey) -> Result<SharedListsPlan, IssuanceError> {
        if !matches!(self.environment.as_str(), "localnet" | "devnet" | "testnet")
            || self.rabovel_admin != admin.to_string()
            || self.gate_program != ABL_GATE_PROGRAM_ID.to_string()
        {
            return Err(IssuanceError::InvalidConfiguration(
                "deployment must name localnet/devnet/testnet and match the configured admin and supported ABL Gate"
                    .into(),
            ));
        }
        let parse_seed = |value: &str| {
            Pubkey::from_str(value).map_err(|_| {
                IssuanceError::InvalidConfiguration("list seed must be a base58 public key".into())
            })
        };
        let seeds = SharedListSeeds {
            allow: parse_seed(&self.allow_list_seed)?,
            block: parse_seed(&self.block_list_seed)?,
        };
        SharedListsBuilder::new(admin)?.build(seeds, admin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_configuration_preserves_addresses_and_cannot_be_overwritten() {
        let admin = Pubkey::new_unique();
        let config = AclDeploymentConfig::new_local(admin);
        let path = std::env::temp_dir().join(format!("rabovel-acl-{}.json", uuid::Uuid::new_v4()));
        config.save_new(&path).unwrap();
        let loaded = AclDeploymentConfig::from_file(&path).unwrap();
        let overwrite = config.save_new(&path);
        std::fs::remove_file(&path).unwrap();
        assert!(overwrite.is_err());
        assert_eq!(
            config.plan(admin).unwrap().addresses,
            loaded.plan(admin).unwrap().addresses
        );
        assert!(loaded.plan(Pubkey::new_unique()).is_err());
        let mut invalid = loaded;
        invalid.gate_program = Pubkey::new_unique().to_string();
        assert!(invalid.plan(admin).is_err());
        invalid.gate_program = ABL_GATE_PROGRAM_ID.to_string();
        invalid.environment = "mainnet-beta".into();
        assert!(invalid.plan(admin).is_err());
    }
}
