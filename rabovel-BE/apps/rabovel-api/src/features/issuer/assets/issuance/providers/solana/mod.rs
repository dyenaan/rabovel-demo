use async_trait::async_trait;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::issuance::inventory::ports::InventoryProvider;
use issuer_assets::assets::issuance::setup::models::AssetSetupOperation;
use issuer_assets::assets::issuance::setup::ports::MintSetupExecutor;
use issuer_assets::assets::positions::ports::PositionProvider;
use sha2::Digest;
use sha2::Sha256;
use solana_adapter::authority_config::SignerConfig;
use solana_adapter::token_acl::AclDeploymentConfig;
use solana_keypair::Keypair;
use solana_signer::Signer;
use std::sync::Arc;

pub(crate) use issuer_assets::assets::issuance::inventory::models::InitialInventory;
pub(crate) use issuer_assets::assets::issuance::setup::models::SetupExecution;
pub(crate) use issuer_assets::assets::issuance::setup::models::SetupExecutionError;
pub(crate) use issuer_assets::assets::positions::models::InvestorAssetPosition;

pub struct SolanaMintSetupExecutor {
    rpc_url: String,
    network: String,
    signer_config: Arc<SignerConfig>,
    acl_config: Arc<AclDeploymentConfig>,
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
            signer_config: Arc::new(signer_config),
            acl_config: Arc::new(acl_config),
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
    pub(crate) fn purchase_configuration(
        &self,
    ) -> (String, Arc<SignerConfig>, Arc<AclDeploymentConfig>) {
        (
            self.rpc_url.clone(),
            self.signer_config.clone(),
            self.acl_config.clone(),
        )
    }
}

pub(crate) struct DisabledMintSetupExecutor;

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
}

#[async_trait]
impl InventoryProvider for DisabledMintSetupExecutor {
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
}

#[async_trait]
impl PositionProvider for DisabledMintSetupExecutor {
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
}

mod inventory;
mod positions;
mod setup;
