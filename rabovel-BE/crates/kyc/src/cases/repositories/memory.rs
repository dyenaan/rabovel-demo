use super::super::models::*;
use super::super::ports::CaseRepository;
use async_trait::async_trait;
use identity::AuthError;
use identity::RiskTier;
use std::collections::HashMap;
use std::sync::RwLock;

fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("identity repository unavailable".into())
}
#[derive(Default)]
pub struct InMemoryCases {
    kyc_cases: RwLock<HashMap<String, KycCaseRecord>>,
}
#[async_trait]
impl CaseRepository for InMemoryCases {
    async fn create_kyc_case(&self, case: KycCaseRecord) -> Result<(), AuthError> {
        self.kyc_cases
            .write()
            .map_err(|_| state_unavailable())?
            .insert(case.case_id.clone(), case);
        Ok(())
    }
    async fn finalize_kyc_case(
        &self,
        case_id: &str,
        status: KycCaseStatus,
        risk_tier: Option<RiskTier>,
        finalized_at: u64,
    ) -> Result<(), AuthError> {
        let mut cases = self.kyc_cases.write().map_err(|_| state_unavailable())?;
        let case = cases
            .get_mut(case_id)
            .ok_or_else(|| AuthError::PolicyViolation("kyc case not found".to_string()))?;
        case.status = status;
        case.risk_tier = risk_tier;
        case.finalized_at = Some(finalized_at);
        Ok(())
    }
    async fn get_kyc_case(&self, case_id: &str) -> Result<Option<KycCaseRecord>, AuthError> {
        Ok(self
            .kyc_cases
            .read()
            .map_err(|_| state_unavailable())?
            .get(case_id)
            .cloned())
    }
}
