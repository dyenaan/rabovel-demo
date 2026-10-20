use crate::*;
use async_trait::async_trait;
use hmac::Hmac;
use hmac::Mac;
use identity::RiskTier;
use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::RwLock;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
struct SandboxCase {
    user_id: String,
    country: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SandboxRiskTier {
    Standard,
    HighRisk,
}
impl From<SandboxRiskTier> for RiskTier {
    fn from(value: SandboxRiskTier) -> Self {
        match value {
            SandboxRiskTier::Standard => RiskTier::Standard,
            SandboxRiskTier::HighRisk => RiskTier::HighRisk,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct WebhookPayload {
    case_id: String,
    user_id: String,
    approved: bool,
    risk_tier: SandboxRiskTier,
    checked_at: u64,
}
/// Sandbox/mock KYC adapter. No real vendor integration -- deterministic
/// rules only, so integration tests and local/QA environments can exercise
/// both an approval and a rejection without any network call.
#[derive(Debug)]
pub struct MockKycProvider {
    shared_secret: Vec<u8>,
    cases: RwLock<HashMap<String, SandboxCase>>,
}
impl MockKycProvider {
    /// This magic country code deterministically forces a rejected/high-risk
    /// verdict so tests can exercise both outcomes; every other country is
    /// approved.
    pub const REJECTED_SANDBOX_COUNTRY: &'static str = "ZZ";

    pub fn new(shared_secret: impl Into<Vec<u8>>) -> Self {
        Self {
            shared_secret: shared_secret.into(),
            cases: RwLock::new(HashMap::new()),
        }
    }

    fn sign(&self, body: &[u8]) -> Result<String, KycError> {
        let mut mac = HmacSha256::new_from_slice(&self.shared_secret)
            .map_err(|_| KycError::Unavailable("invalid webhook secret".to_string()))?;
        mac.update(body);
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    /// Test/dev-only helper: builds the raw webhook body + signature the
    /// sandbox would send as its own asynchronous callback for `case_id`, so
    /// local/QA environments can simulate the vendor's callback without a
    /// real HTTP round trip. A `cfg(debug_assertions)`-only gateway route
    /// drives this through the same `verify_webhook` code path a real
    /// vendor's callback would hit.
    pub fn force_webhook_payload(&self, case_id: &str) -> Result<(Vec<u8>, String), KycError> {
        let case = {
            let cases = self
                .cases
                .read()
                .map_err(|_| KycError::Unavailable("case store poisoned".to_string()))?;
            cases.get(case_id).cloned().ok_or(KycError::CaseNotFound)?
        };
        let approved = case.country != Self::REJECTED_SANDBOX_COUNTRY;
        let payload = WebhookPayload {
            case_id: case_id.to_string(),
            user_id: case.user_id,
            approved,
            risk_tier: if approved {
                SandboxRiskTier::Standard
            } else {
                SandboxRiskTier::HighRisk
            },
            checked_at: unix_now(),
        };
        let body = serde_json::to_vec(&payload).map_err(|_| KycError::MalformedPayload)?;
        let signature = self.sign(&body)?;
        Ok((body, signature))
    }
}
#[async_trait]
impl KycProviderApi for MockKycProvider {
    async fn submit_case(&self, submission: KycSubmission) -> Result<KycCaseHandle, KycError> {
        let case_id = format!("kyc_{}", random_hex(16)?);
        let submitted_at = unix_now();
        self.cases
            .write()
            .map_err(|_| KycError::Unavailable("case store poisoned".to_string()))?
            .insert(
                case_id.clone(),
                SandboxCase {
                    user_id: submission.user_id,
                    country: submission.country,
                },
            );
        Ok(KycCaseHandle {
            case_id,
            submitted_at,
        })
    }

    fn verify_webhook(
        &self,
        raw_body: &[u8],
        signature_header: &str,
    ) -> Result<VerifiedKycVerdict, KycError> {
        let expected = self.sign(raw_body)?;
        let provided = signature_header
            .strip_prefix("sha256=")
            .unwrap_or(signature_header);
        // Length is checked before the constant-time comparison so `ct_eq`
        // (which only compares equal-length slices) never panics; this
        // leaks the signature's length via timing, not its content, which
        // is the standard accepted tradeoff for HMAC webhook verification.
        if expected.len() != provided.len()
            || !bool::from(expected.as_bytes().ct_eq(provided.as_bytes()))
        {
            return Err(KycError::InvalidSignature);
        }
        let payload: WebhookPayload =
            serde_json::from_slice(raw_body).map_err(|_| KycError::MalformedPayload)?;
        Ok(VerifiedKycVerdict {
            case_id: payload.case_id,
            user_id: payload.user_id,
            kyc_verified: payload.approved,
            phone_verified: payload.approved,
            risk_tier: payload.risk_tier.into(),
            checked_at: payload.checked_at,
        })
    }
}
fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
fn random_hex(bytes: usize) -> Result<String, KycError> {
    let mut value = vec![0u8; bytes];
    getrandom::getrandom(&mut value)
        .map_err(|_| KycError::Unavailable("OS random source unavailable".to_string()))?;
    Ok(hex::encode(value))
}
