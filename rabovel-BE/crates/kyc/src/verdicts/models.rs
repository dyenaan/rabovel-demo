use identity::AuthError;
use identity::RiskTier;
use identity::VerifiedComplianceEvidence;

/// Provider output produced after webhook signature validation. Application
/// workflows accept this evidence through the KYC provider contract.
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
    /// Bridges into `identity`'s designed KYC-output contract. A KYC
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
