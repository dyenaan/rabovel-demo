use super::*;
use async_trait::async_trait;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::issuance::setup::models::AssetSetupOperation;
use solana_adapter::token_acl::MintAclService;
use solana_adapter::token_acl::SharedListsService;
use solana_adapter::token_mint_builder::TokenMetadata;
use solana_adapter::EquitySetupService;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_signer::Signer;

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
}
