use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct SetupStageResponse {
    pub stage: String,
    pub status: String,
    pub signatures: Vec<String>,
    pub verified_at: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AssetSetupOperationResponse {
    pub operation_id: String,
    pub asset_id: String,
    pub network: String,
    pub mint_address: String,
    pub status: String,
    pub stages: Vec<SetupStageResponse>,
    pub mint_config_address: Option<String>,
    pub allow_list_address: Option<String>,
    pub block_list_address: Option<String>,
    pub thaw_extra_metas_address: Option<String>,
    #[serde(serialize_with = "crate::http::observability::serialize_provider_error")]
    pub error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}
