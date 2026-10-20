use kyc::DocumentType;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KycCaseRequest {
    pub full_name: String,
    pub date_of_birth: String,
    pub country: String,
    pub document_type: DocumentType,
    pub document_reference: String,
}

#[derive(Debug, Serialize)]
pub struct KycCaseResponse {
    pub case_id: String,
    pub submitted_at: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg(debug_assertions)]
pub(crate) struct SandboxAdvanceRequest {
    pub(crate) case_id: String,
}
