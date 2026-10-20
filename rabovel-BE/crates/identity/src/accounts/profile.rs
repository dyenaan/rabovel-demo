use crate::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum BrokerageOnboardingStatus {
    Approved,
    RequiresReview,
    Blocked,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AuthProvider {
    EmailPassword,
    Google,
    Wallet,
    Passkey,
}

/// Claims produced by a trusted OpenID Connect verifier. This type deliberately
/// does not implement `Deserialize`, so an HTTP client cannot manufacture it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedGoogleIdentity {
    pub(crate) issuer: String,
    pub(crate) subject: String,
    pub(crate) email: String,
}

impl VerifiedGoogleIdentity {
    pub fn from_verified_claims(
        issuer: impl Into<String>,
        subject: impl Into<String>,
        email: impl Into<String>,
        email_verified: bool,
    ) -> Result<Self, AuthError> {
        let issuer = issuer.into();
        let subject = subject.into();
        let email = email.into();
        if !email_verified {
            return Err(AuthError::UnverifiedEmail);
        }
        if issuer.trim().is_empty() || subject.trim().is_empty() || email.trim().is_empty() {
            return Err(AuthError::InvalidIdentityProof);
        }
        Ok(Self {
            issuer,
            subject,
            email,
        })
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    pub fn subject(&self) -> &str {
        &self.subject
    }
    pub fn email(&self) -> &str {
        &self.email
    }
}

/// Evidence accepted only from a trusted KYC/AML adapter or verified signed webhook.
/// It is intentionally not deserializable from a public request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedComplianceEvidence {
    pub(crate) phone_verified: bool,
    pub(crate) kyc_verified: bool,
    pub(crate) mfa_enrolled: bool,
    pub(crate) trusted_device: bool,
    pub(crate) risk_tier: RiskTier,
    pub(crate) checked_at: u64,
}

impl VerifiedComplianceEvidence {
    pub fn new(
        phone_verified: bool,
        kyc_verified: bool,
        mfa_enrolled: bool,
        trusted_device: bool,
        risk_tier: RiskTier,
        checked_at: u64,
    ) -> Result<Self, AuthError> {
        if matches!(risk_tier, RiskTier::Unassessed) {
            return Err(AuthError::PolicyViolation(
                "verified compliance evidence must contain an assessed risk tier".to_string(),
            ));
        }
        Ok(Self {
            phone_verified,
            kyc_verified,
            mfa_enrolled,
            trusted_device,
            risk_tier,
            checked_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub struct IdentityProfile {
    pub(crate) user_id: String,
    pub(crate) email: String,
    pub(crate) email_verified: bool,
    pub(crate) phone_verified: bool,
    pub(crate) kyc_verified: bool,
    pub(crate) terms_version: Option<String>,
    pub(crate) mfa_enrolled: bool,
    pub(crate) trusted_device: bool,
    pub(crate) risk_tier: RiskTier,
    pub(crate) compliance_checked_at: Option<u64>,
    pub(crate) wallet_ids: Vec<String>,
    pub(crate) auth_provider: AuthProvider,
}

impl IdentityProfile {
    pub(crate) fn from_google(
        user_id: impl Into<String>,
        identity: &VerifiedGoogleIdentity,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            email: identity.email().to_string(),
            email_verified: true,
            phone_verified: false,
            kyc_verified: false,
            terms_version: None,
            mfa_enrolled: false,
            trusted_device: false,
            risk_tier: RiskTier::Unassessed,
            compliance_checked_at: None,
            wallet_ids: Vec::new(),
            auth_provider: AuthProvider::Google,
        }
    }

    pub(crate) fn from_email_password(
        user_id: impl Into<String>,
        email: impl Into<String>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            email: email.into(),
            email_verified: false,
            phone_verified: false,
            kyc_verified: false,
            terms_version: None,
            mfa_enrolled: false,
            trusted_device: false,
            risk_tier: RiskTier::Unassessed,
            compliance_checked_at: None,
            wallet_ids: Vec::new(),
            auth_provider: AuthProvider::EmailPassword,
        }
    }

    /// Rehydrates a profile from durable storage (e.g. a Postgres row already
    /// written by this same process). This is **not** a general-purpose
    /// constructor: it must never be fed client-supplied input, since it
    /// bypasses every verification step that `from_google` and
    /// `apply_verified_compliance` normally enforce. Callers are trusted to
    /// only pass back values this process previously persisted itself.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        user_id: impl Into<String>,
        email: impl Into<String>,
        email_verified: bool,
        phone_verified: bool,
        kyc_verified: bool,
        terms_version: Option<String>,
        mfa_enrolled: bool,
        trusted_device: bool,
        risk_tier: RiskTier,
        compliance_checked_at: Option<u64>,
        wallet_ids: Vec<String>,
        auth_provider: AuthProvider,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            email: email.into(),
            email_verified,
            phone_verified,
            kyc_verified,
            terms_version,
            mfa_enrolled,
            trusted_device,
            risk_tier,
            compliance_checked_at,
            wallet_ids,
            auth_provider,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn email_verified(&self) -> bool {
        self.email_verified
    }
    pub fn phone_verified(&self) -> bool {
        self.phone_verified
    }
    pub fn kyc_verified(&self) -> bool {
        self.kyc_verified
    }
    pub fn terms_version(&self) -> Option<&str> {
        self.terms_version.as_deref()
    }
    pub fn risk_tier(&self) -> RiskTier {
        self.risk_tier
    }
    pub fn mfa_enrolled(&self) -> bool {
        self.mfa_enrolled
    }
    pub fn trusted_device(&self) -> bool {
        self.trusted_device
    }
    pub fn compliance_checked_at(&self) -> Option<u64> {
        self.compliance_checked_at
    }
    pub fn wallet_ids(&self) -> &[String] {
        &self.wallet_ids
    }
    pub fn auth_provider(&self) -> AuthProvider {
        self.auth_provider
    }

    pub fn apply_verified_compliance(&mut self, evidence: VerifiedComplianceEvidence) {
        self.phone_verified = evidence.phone_verified;
        self.kyc_verified = evidence.kyc_verified;
        self.mfa_enrolled = evidence.mfa_enrolled;
        self.trusted_device = evidence.trusted_device;
        self.risk_tier = evidence.risk_tier;
        self.compliance_checked_at = Some(evidence.checked_at);
    }

    pub fn accept_terms(&mut self, version: impl Into<String>) -> Result<(), AuthError> {
        let version = version.into();
        if version.trim().is_empty() {
            return Err(AuthError::PolicyViolation(
                "a non-empty terms version is required".to_string(),
            ));
        }
        self.terms_version = Some(version);
        Ok(())
    }

    pub fn record_verified_wallet(&mut self, wallet_id: impl Into<String>) {
        let wallet_id = wallet_id.into();
        if !self.wallet_ids.contains(&wallet_id) {
            self.wallet_ids.push(wallet_id);
        }
    }
}
