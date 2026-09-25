//! Pluggable KYC/AML verification layer.
//!
//! [`KycProviderApi`] is the seam a real vendor (Smile Identity, Youverify,
//! Dojah, Sumsub, ...) is wired in behind later; only [`MockKycProvider`]'s
//! `submit_case` outbound call and `verify_webhook` signature scheme would
//! change for a real vendor -- the trait shape and everything calling it
//! (the BFF's KYC routes) does not.
//!
//! [`VerifiedKycVerdict::into_compliance_evidence`] is the bridge into
//! `domain::auth`'s designed KYC-output contract
//! (`domain::auth::VerifiedComplianceEvidence`).

use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use domain::auth::{AuthError, RiskTier, VerifiedComplianceEvidence};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum KycError {
    #[error("the KYC case was not found")]
    CaseNotFound,
    #[error("the webhook signature is invalid")]
    InvalidSignature,
    #[error("the webhook payload could not be parsed")]
    MalformedPayload,
    #[error("the KYC provider is unavailable: {0}")]
    Unavailable(String),
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

/// Constructed only inside [`KycProviderApi::verify_webhook`] after
/// signature validation passes -- the same "server-only-constructible
/// verified evidence" pattern `domain::auth` uses for
/// `VerifiedGoogleIdentity`/`VerifiedComplianceEvidence`. A client can never
/// manufacture one directly.
#[derive(Debug, Clone)]
pub struct VerifiedKycVerdict {
    pub case_id: String,
    pub user_id: String,
    pub kyc_verified: bool,
    pub phone_verified: bool,
    pub risk_tier: RiskTier,
    pub checked_at: u64,
}

impl VerifiedKycVerdict {
    /// Bridges into `domain::auth`'s designed KYC-output contract. A KYC
    /// vendor has no notion of MFA enrollment or trusted-device status, so
    /// the caller (the BFF, which already knows the user's current values)
    /// supplies those two separately.
    pub fn into_compliance_evidence(
        self,
        mfa_enrolled: bool,
        trusted_device: bool,
    ) -> Result<VerifiedComplianceEvidence, AuthError> {
        VerifiedComplianceEvidence::new(
            self.phone_verified,
            self.kyc_verified,
            mfa_enrolled,
            trusted_device,
            self.risk_tier,
            self.checked_at,
        )
    }
}

#[async_trait]
pub trait KycProviderApi: Send + Sync {
    async fn submit_case(&self, submission: KycSubmission) -> Result<KycCaseHandle, KycError>;

    /// Verifies an inbound webhook's authenticity over the *raw* body bytes
    /// before any structured parsing occurs, then returns a verdict.
    /// Callers must pass the untouched request body -- never a
    /// re-serialized/re-parsed form of it -- since the signature covers the
    /// exact bytes the vendor sent.
    fn verify_webhook(
        &self,
        raw_body: &[u8],
        signature_header: &str,
    ) -> Result<VerifiedKycVerdict, KycError>;
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn provider() -> MockKycProvider {
        MockKycProvider::new(b"test-shared-secret".to_vec())
    }

    #[tokio::test]
    async fn approved_case_round_trips_into_compliance_evidence() {
        let provider = provider();
        let handle = provider
            .submit_case(KycSubmission {
                user_id: "user-1".to_string(),
                full_name: "Ada Lovelace".to_string(),
                date_of_birth: "1990-01-01".to_string(),
                country: "NG".to_string(),
                document_type: DocumentType::Passport,
                document_reference: "vendor-ref-123".to_string(),
                request_id: "req-1".to_string(),
            })
            .await
            .unwrap();

        let (body, signature) = provider.force_webhook_payload(&handle.case_id).unwrap();
        let verdict = provider.verify_webhook(&body, &signature).unwrap();
        assert_eq!(verdict.case_id, handle.case_id);
        assert_eq!(verdict.user_id, "user-1");
        assert!(verdict.kyc_verified);
        assert_eq!(verdict.risk_tier, RiskTier::Standard);

        let evidence = verdict.into_compliance_evidence(true, true).unwrap();
        // VerifiedComplianceEvidence's fields are private by design; the
        // only observable proof of construction is that `new` succeeded
        // (it rejects `RiskTier::Unassessed`), which this assert confirms.
        let _ = evidence;
    }

    #[tokio::test]
    async fn sandbox_country_forces_a_rejected_verdict() {
        let provider = provider();
        let handle = provider
            .submit_case(KycSubmission {
                user_id: "user-2".to_string(),
                full_name: "Attempted Fraud".to_string(),
                date_of_birth: "1990-01-01".to_string(),
                country: MockKycProvider::REJECTED_SANDBOX_COUNTRY.to_string(),
                document_type: DocumentType::NationalId,
                document_reference: "vendor-ref-456".to_string(),
                request_id: "req-2".to_string(),
            })
            .await
            .unwrap();

        let (body, signature) = provider.force_webhook_payload(&handle.case_id).unwrap();
        let verdict = provider.verify_webhook(&body, &signature).unwrap();
        assert!(!verdict.kyc_verified);
        assert_eq!(verdict.risk_tier, RiskTier::HighRisk);
    }

    #[tokio::test]
    async fn tampered_webhook_body_is_rejected() {
        let provider = provider();
        let handle = provider
            .submit_case(KycSubmission {
                user_id: "user-3".to_string(),
                full_name: "Tamper Test".to_string(),
                date_of_birth: "1990-01-01".to_string(),
                country: "NG".to_string(),
                document_type: DocumentType::DriversLicense,
                document_reference: "vendor-ref-789".to_string(),
                request_id: "req-3".to_string(),
            })
            .await
            .unwrap();

        let (mut body, signature) = provider.force_webhook_payload(&handle.case_id).unwrap();
        // Flip the `approved` verdict in the raw bytes without re-signing --
        // simulates a network-path tamper attempt.
        let tampered = String::from_utf8(body.clone())
            .unwrap()
            .replace("\"approved\":true", "\"approved\":false");
        body = tampered.into_bytes();
        let result = provider.verify_webhook(&body, &signature);
        assert!(matches!(result, Err(KycError::InvalidSignature)));
    }

    #[test]
    fn unknown_case_id_is_rejected() {
        let provider = provider();
        let result = provider.force_webhook_payload("kyc_does_not_exist");
        assert!(matches!(result, Err(KycError::CaseNotFound)));
    }
}
