use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageUploadRequest {
    pub content_type: String,
    pub size: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImageUploadTicket {
    pub signed_url: String,
    pub object_path: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfirmImageUploadRequest {
    pub object_path: String,
    pub content_type: String,
}
