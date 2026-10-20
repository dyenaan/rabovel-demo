use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SubmitBackingRequest {
    pub summary: String,
    pub document_name: String,
    pub content_type: String,
    pub size_bytes: u64,
}
