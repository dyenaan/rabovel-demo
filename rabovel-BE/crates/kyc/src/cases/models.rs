use identity::RiskTier;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KycCaseStatus {
    Submitted,
    Approved,
    Rejected,
    ManualReview,
}
impl KycCaseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::ManualReview => "manual_review",
        }
    }
}
impl std::str::FromStr for KycCaseStatus {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "submitted" => Ok(Self::Submitted),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "manual_review" => Ok(Self::ManualReview),
            _ => Err(()),
        }
    }
}
#[derive(Debug, Clone)]
pub struct KycCaseRecord {
    pub case_id: String,
    pub user_id: String,
    pub status: KycCaseStatus,
    pub risk_tier: Option<RiskTier>,
    pub submitted_at: u64,
    pub finalized_at: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Passport,
    NationalId,
    DriversLicense,
}
/// A pointer into a client-side vendor SDK/widget upload -- never raw
/// document bytes. This keeps the BFF out of multipart upload / virus
/// scanning territory entirely, matching how real KYC vendors actually
/// integrate (the document itself is uploaded directly to the vendor by
/// the client widget; the backend only ever sees a reference to it).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycSubmission {
    pub user_id: String,
    pub full_name: String,
    pub date_of_birth: String,
    pub country: String,
    pub document_type: DocumentType,
    pub document_reference: String,
    pub request_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycCaseHandle {
    pub case_id: String,
    pub submitted_at: u64,
}
