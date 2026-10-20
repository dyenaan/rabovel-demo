use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct IssuerOverviewResponse {
    pub user_id: String,
    pub email: String,
    pub account_status: String,
    pub organization: Option<IssuerOrganizationResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IssuerOrganizationResponse {
    pub legal_name: String,
    pub organization_type: String,
    pub registration_number: String,
    pub jurisdiction: String,
    pub registered_address: String,
    pub representative_name: String,
    pub representative_title: String,
    pub document_reference: String,
    pub approved_at: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct IssuerOnboardingRequest {
    pub legal_name: String,
    pub organization_type: String,
    pub registration_number: String,
    pub jurisdiction: String,
    pub registered_address: String,
    pub representative_name: String,
    pub representative_title: String,
    pub document_reference: String,
    pub beneficial_owners_confirmed: bool,
    pub information_certified: bool,
}
