use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetupStageRecord {
    pub stage: String,
    pub status: String,
    pub signatures: Vec<String>,
    pub verified_at: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetSetupOperation {
    pub operation_id: String,
    pub asset_id: String,
    pub issuer_user_id: String,
    pub network: String,
    pub mint_address: String,
    pub status: String,
    pub stages: Vec<SetupStageRecord>,
    pub mint_config_address: Option<String>,
    pub allow_list_address: Option<String>,
    pub block_list_address: Option<String>,
    pub thaw_extra_metas_address: Option<String>,
    pub error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug)]
pub struct SetupExecution {
    pub mint_signature: Option<String>,
    pub shared_lists_signature: Option<String>,
    pub acl_signatures: Vec<String>,
    pub mint_config_address: String,
    pub allow_list_address: String,
    pub block_list_address: String,
    pub thaw_extra_metas_address: String,
}

#[derive(Debug)]
pub struct SetupExecutionError {
    pub message: String,
    pub reconciliation_required: bool,
}
