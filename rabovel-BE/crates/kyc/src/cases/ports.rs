use super::models::*;
use async_trait::async_trait;
use identity::AuthError;
use identity::RiskTier;

#[async_trait]
pub trait CaseRepository: Send + Sync {
    async fn create_kyc_case(&self, case: KycCaseRecord) -> Result<(), AuthError>;

    async fn finalize_kyc_case(
        &self,
        case_id: &str,
        status: KycCaseStatus,
        risk_tier: Option<RiskTier>,
        finalized_at: u64,
    ) -> Result<(), AuthError>;
    async fn get_kyc_case(&self, case_id: &str) -> Result<Option<KycCaseRecord>, AuthError>;
}
