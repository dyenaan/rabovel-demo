#[derive(Debug, Clone)]
pub struct IssuerOrganization {
    pub user_id: String,
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
