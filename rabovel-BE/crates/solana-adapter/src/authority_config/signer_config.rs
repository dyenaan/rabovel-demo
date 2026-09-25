use super::AuthConfig;
use serde::Deserialize;
use solana_keypair::{Keypair, read_keypair_file};
use solana_signer::Signer;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not read authority configuration")]
    ReadConfig(#[source] std::io::Error),
    #[error("invalid authority configuration JSON")]
    InvalidConfig(#[source] serde_json::Error),
    #[error("issuer is not configured: {0}")]
    UnknownIssuer(String),
    #[error("could not load {role} keypair from {path}")]
    FileKeypair { role: &'static str, path: PathBuf },
    #[error("invalid {role} keypair secret")]
    SecretKeypair { role: &'static str },
}

#[derive(Debug)]
enum KeypairSource {
    File(PathBuf),
    Secret(Vec<u8>),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileSignerConfig {
    rabovel_admin_keypair: PathBuf,
    broker_keypair: PathBuf,
    issuers: BTreeMap<String, PathBuf>,
}

/// Service-controlled signer sources. Hosted runtimes use validated in-memory
/// secrets; file sources remain available for local commands and runbooks.
#[derive(Debug)]
pub struct SignerConfig {
    rabovel_admin: KeypairSource,
    broker: KeypairSource,
    default_issuer: Option<KeypairSource>,
    issuers: BTreeMap<String, KeypairSource>,
}

pub struct AuthoritySigners {
    pub rabovel_admin: Keypair,
    pub issuer: Keypair,
}

impl AuthoritySigners {
    pub fn auth_config(&self) -> AuthConfig {
        AuthConfig {
            rabovel_admin: self.rabovel_admin.pubkey(),
            issuer: self.issuer.pubkey(),
        }
    }
}

impl SignerConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let config_path = path.as_ref();
        let bytes = std::fs::read(config_path).map_err(ConfigError::ReadConfig)?;
        let config: FileSignerConfig =
            serde_json::from_slice(&bytes).map_err(ConfigError::InvalidConfig)?;
        let directory = config_path.parent().unwrap_or_else(|| Path::new("."));
        Ok(Self {
            rabovel_admin: KeypairSource::File(directory.join(config.rabovel_admin_keypair)),
            broker: KeypairSource::File(directory.join(config.broker_keypair)),
            default_issuer: None,
            issuers: config
                .issuers
                .into_iter()
                .map(|(id, path)| (id, KeypairSource::File(directory.join(path))))
                .collect(),
        })
    }

    pub fn from_secret_bytes(
        admin: Vec<u8>,
        broker: Vec<u8>,
        issuer: Vec<u8>,
    ) -> Result<Self, ConfigError> {
        validate_secret(&admin, "Rabovel Admin")?;
        validate_secret(&broker, "broker")?;
        validate_secret(&issuer, "issuer")?;
        Ok(Self {
            rabovel_admin: KeypairSource::Secret(admin),
            broker: KeypairSource::Secret(broker),
            default_issuer: Some(KeypairSource::Secret(issuer)),
            issuers: BTreeMap::new(),
        })
    }

    pub fn load_admin(&self) -> Result<Keypair, ConfigError> {
        load_keypair(&self.rabovel_admin, "Rabovel Admin")
    }
    pub fn load_broker(&self) -> Result<Keypair, ConfigError> {
        load_keypair(&self.broker, "broker")
    }
    pub fn broker_pubkey(&self) -> Result<String, ConfigError> {
        Ok(self.load_broker()?.pubkey().to_string())
    }

    pub fn load(&self, issuer_id: &str) -> Result<AuthoritySigners, ConfigError> {
        let issuer = self
            .issuers
            .get(issuer_id)
            .or(self.default_issuer.as_ref())
            .ok_or_else(|| ConfigError::UnknownIssuer(issuer_id.to_owned()))?;
        Ok(AuthoritySigners {
            rabovel_admin: self.load_admin()?,
            issuer: load_keypair(issuer, "issuer")?,
        })
    }
}

fn validate_secret(bytes: &[u8], role: &'static str) -> Result<(), ConfigError> {
    Keypair::try_from(bytes)
        .map(|_| ())
        .map_err(|_| ConfigError::SecretKeypair { role })
}

fn load_keypair(source: &KeypairSource, role: &'static str) -> Result<Keypair, ConfigError> {
    match source {
        KeypairSource::File(path) => {
            read_keypair_file(path).map_err(|_| ConfigError::FileKeypair {
                role,
                path: path.clone(),
            })
        }
        KeypairSource::Secret(bytes) => {
            Keypair::try_from(bytes.as_slice()).map_err(|_| ConfigError::SecretKeypair { role })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_keypair::write_keypair_file;

    #[test]
    fn file_and_secret_sources_load_all_authorities_without_leaking_secrets() {
        let directory =
            std::env::temp_dir().join(format!("rabovel-config-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let result = std::panic::catch_unwind(|| {
            let admin = Keypair::new();
            let broker = Keypair::new();
            let issuer = Keypair::new();
            write_keypair_file(&admin, directory.join("admin.json")).unwrap();
            write_keypair_file(&broker, directory.join("broker.json")).unwrap();
            write_keypair_file(&issuer, directory.join("issuer.json")).unwrap();
            let config_path = directory.join("config.json");
            std::fs::write(&config_path, r#"{"rabovel_admin_keypair":"admin.json","broker_keypair":"broker.json","issuers":{"demo":"issuer.json"}}"#).unwrap();
            let files = SignerConfig::from_file(config_path).unwrap();
            assert_eq!(
                files.load("demo").unwrap().auth_config().issuer,
                issuer.pubkey()
            );
            assert_eq!(files.broker_pubkey().unwrap(), broker.pubkey().to_string());
            let secrets = SignerConfig::from_secret_bytes(
                admin.to_bytes().to_vec(),
                broker.to_bytes().to_vec(),
                issuer.to_bytes().to_vec(),
            )
            .unwrap();
            assert_eq!(secrets.load_admin().unwrap().pubkey(), admin.pubkey());
            assert_eq!(secrets.load_broker().unwrap().pubkey(), broker.pubkey());
            let error = SignerConfig::from_secret_bytes(
                vec![7; 64],
                broker.to_bytes().to_vec(),
                issuer.to_bytes().to_vec(),
            )
            .unwrap_err();
            assert!(!error.to_string().contains("7, 7"));
        });
        std::fs::remove_dir_all(directory).unwrap();
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }
}
