use serde::{Deserialize, Serialize};

use crate::{envelope::DomainEvent, topics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycCaseSubmitted {
    pub case_id: String,
    pub user_id: String,
    pub submitted_at: u64,
}

impl DomainEvent for KycCaseSubmitted {
    const EVENT_TYPE: &'static str = "kyc.case_submitted";
    const TOPIC: &'static str = topics::KYC_CASE_SUBMITTED_V1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KycRiskTier {
    Standard,
    Enhanced,
    HighRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycVerdictReceived {
    pub case_id: String,
    pub user_id: String,
    pub kyc_verified: bool,
    pub risk_tier: KycRiskTier,
    pub checked_at: u64,
}

impl DomainEvent for KycVerdictReceived {
    const EVENT_TYPE: &'static str = "kyc.verdict_received";
    const TOPIC: &'static str = topics::KYC_VERDICT_RECEIVED_V1;
}
