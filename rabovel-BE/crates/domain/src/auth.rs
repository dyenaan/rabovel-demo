// -----------------------------------------------------------------------------
// Active authentication and authorization domain model.
// -----------------------------------------------------------------------------

use std::collections::HashSet;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Trader,
    Issuer,
    ComplianceOfficer,
    Admin,
    RiskOps,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Unassessed,
    Standard,
    Enhanced,
    HighRisk,
}

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
pub enum WalletProvider {
    MetaMask,
    WalletConnect,
    Coinbase,
    Phantom,
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
pub enum WalletAction {
    #[serde(rename = "view:portfolio")]
    ViewPortfolio,
    #[serde(rename = "trade:spot")]
    TradeSpot,
    #[serde(rename = "trade:margin")]
    TradeMargin,
    #[serde(rename = "sign:withdrawal")]
    SignWithdrawal,
    #[serde(rename = "stake:yield")]
    StakeYield,
}

impl WalletAction {
    pub fn is_financial(self) -> bool {
        !matches!(self, Self::ViewPortfolio)
    }

    pub fn requires_fresh_mfa(self) -> bool {
        matches!(
            self,
            Self::TradeMargin | Self::SignWithdrawal | Self::StakeYield
        )
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SupportedChain {
    Ethereum,
    Base,
    Polygon,
    Solana,
}

impl SupportedChain {
    pub fn from_eip155_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            1 => Some(Self::Ethereum),
            8453 => Some(Self::Base),
            137 => Some(Self::Polygon),
            _ => None,
        }
    }

    pub fn eip155_chain_id(self) -> Option<u64> {
        match self {
            Self::Ethereum => Some(1),
            Self::Base => Some(8453),
            Self::Polygon => Some(137),
            Self::Solana => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    InvalidProvider,
    UnverifiedEmail,
    InvalidIdentityProof,
    Unauthorized,
    SessionExpired,
    MissingWallet,
    WalletNotOwned,
    InvalidWalletProof,
    ChallengeExpired,
    ReplayDetected,
    ForbiddenAction,
    OnboardingIncomplete,
    MfaRequired,
    RateLimited,
    PolicyViolation(String),
}

impl AuthError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidProvider => "invalid_provider",
            Self::UnverifiedEmail => "unverified_email",
            Self::InvalidIdentityProof => "invalid_identity_proof",
            Self::Unauthorized => "unauthorized",
            Self::SessionExpired => "session_expired",
            Self::MissingWallet => "missing_wallet",
            Self::WalletNotOwned => "wallet_not_owned",
            Self::InvalidWalletProof => "invalid_wallet_proof",
            Self::ChallengeExpired => "challenge_expired",
            Self::ReplayDetected => "replayed_challenge",
            Self::ForbiddenAction => "forbidden_action",
            Self::OnboardingIncomplete => "onboarding_incomplete",
            Self::MfaRequired => "mfa_required",
            Self::RateLimited => "rate_limited",
            Self::PolicyViolation(_) => "policy_violation",
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProvider => write!(f, "authentication provider is not enabled"),
            Self::UnverifiedEmail => write!(f, "a verified email is required"),
            Self::InvalidIdentityProof => write!(f, "identity proof could not be verified"),
            Self::Unauthorized => write!(f, "authentication is required"),
            Self::SessionExpired => write!(f, "the authentication session has expired"),
            Self::MissingWallet => write!(f, "a verified wallet is required"),
            Self::WalletNotOwned => write!(f, "the wallet is not owned by this principal"),
            Self::InvalidWalletProof => write!(f, "wallet ownership proof is invalid"),
            Self::ChallengeExpired => write!(f, "the verification challenge has expired"),
            Self::ReplayDetected => write!(f, "the verification challenge has already been used"),
            Self::ForbiddenAction => write!(f, "the requested action is not permitted"),
            Self::OnboardingIncomplete => write!(f, "brokerage onboarding is not approved"),
            Self::MfaRequired => write!(f, "fresh multi-factor authentication is required"),
            Self::RateLimited => write!(f, "too many requests, please slow down"),
            Self::PolicyViolation(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AuthError {}

/// Claims produced by a trusted OpenID Connect verifier. This type deliberately
/// does not implement `Deserialize`, so an HTTP client cannot manufacture it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedGoogleIdentity {
    issuer: String,
    subject: String,
    email: String,
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
    phone_verified: bool,
    kyc_verified: bool,
    mfa_enrolled: bool,
    trusted_device: bool,
    risk_tier: RiskTier,
    checked_at: u64,
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
    user_id: String,
    email: String,
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
}

impl IdentityProfile {
    fn from_google(user_id: impl Into<String>, identity: &VerifiedGoogleIdentity) -> Self {
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

    fn from_email_password(user_id: impl Into<String>, email: impl Into<String>) -> Self {
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

/// Output-only wallet state created after cryptographic proof verification.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub struct WalletConnection {
    wallet_id: String,
    owner_user_id: String,
    chain: SupportedChain,
    address: String,
    provider: WalletProvider,
    verified_at: u64,
}

impl WalletConnection {
    pub fn from_verified_proof(
        wallet_id: impl Into<String>,
        owner_user_id: impl Into<String>,
        chain: SupportedChain,
        address: impl Into<String>,
        provider: WalletProvider,
        verified_at: u64,
    ) -> Result<Self, AuthError> {
        let wallet_id = wallet_id.into();
        let owner_user_id = owner_user_id.into();
        let address = address.into();
        if wallet_id.trim().is_empty() || owner_user_id.trim().is_empty() {
            return Err(AuthError::InvalidWalletProof);
        }
        let address = match chain {
            SupportedChain::Solana => {
                if !matches!(provider, WalletProvider::Phantom)
                    || !(32..=44).contains(&address.len())
                    || address.starts_with("0x")
                {
                    return Err(AuthError::InvalidWalletProof);
                }
                address
            }
            _ => {
                if matches!(provider, WalletProvider::Phantom)
                    || !address.starts_with("0x")
                    || address.len() != 42
                {
                    return Err(AuthError::InvalidWalletProof);
                }
                address.to_ascii_lowercase()
            }
        };
        Ok(Self {
            wallet_id,
            owner_user_id,
            chain,
            address,
            provider,
            verified_at,
        })
    }

    pub fn wallet_id(&self) -> &str {
        &self.wallet_id
    }
    pub fn owner_user_id(&self) -> &str {
        &self.owner_user_id
    }
    pub fn address(&self) -> &str {
        &self.address
    }
    pub fn chain(&self) -> SupportedChain {
        self.chain
    }
    pub fn provider(&self) -> WalletProvider {
        self.provider
    }
    pub fn verified_at(&self) -> u64 {
        self.verified_at
    }
}

/// Server-loaded authorization data. It cannot be deserialized from a client request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AuthPrincipal {
    user_id: String,
    roles: HashSet<UserRole>,
    permissions: HashSet<WalletAction>,
}

impl AuthPrincipal {
    pub fn new(
        user_id: impl Into<String>,
        roles: HashSet<UserRole>,
        permissions: HashSet<WalletAction>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            roles,
            permissions,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    pub fn has_role(&self, role: UserRole) -> bool {
        self.roles.contains(&role)
    }
    pub fn has_permission(&self, permission: WalletAction) -> bool {
        self.permissions.contains(&permission)
    }
    /// For trusted persistence (e.g. writing a Postgres row) -- never for
    /// exposing this data to a client.
    pub fn roles_iter(&self) -> impl Iterator<Item = UserRole> + '_ {
        self.roles.iter().copied()
    }
    /// For trusted persistence (e.g. writing a Postgres row) -- never for
    /// exposing this data to a client.
    pub fn permissions_iter(&self) -> impl Iterator<Item = WalletAction> + '_ {
        self.permissions.iter().copied()
    }

    /// Replaces roles from trusted server configuration. Never call this with
    /// request data: public authentication must not choose authorization.
    pub fn replace_roles(&mut self, roles: HashSet<UserRole>) {
        self.roles = roles;
    }

    /// Grants an additional wallet action permission. Used for role-based
    /// permission grants during user onboarding.
    pub fn grant_permission(&mut self, permission: WalletAction) {
        self.permissions.insert(permission);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerageSecurityPolicy {
    require_email_verification: bool,
    require_phone_verification: bool,
    require_kyc: bool,
    require_mfa_enrollment: bool,
    require_terms_acceptance: bool,
    require_trusted_device: bool,
    require_risk_assessment: bool,
    blocking_risk_tiers: HashSet<RiskTier>,
    allowed_wallet_actions: HashSet<WalletAction>,
    enabled_auth_providers: HashSet<AuthProvider>,
    step_up_max_age_seconds: u64,
}

impl Default for BrokerageSecurityPolicy {
    fn default() -> Self {
        Self {
            require_email_verification: true,
            require_phone_verification: true,
            require_kyc: true,
            require_mfa_enrollment: true,
            require_terms_acceptance: true,
            require_trusted_device: true,
            require_risk_assessment: true,
            blocking_risk_tiers: HashSet::from([RiskTier::HighRisk]),
            allowed_wallet_actions: HashSet::from([
                WalletAction::ViewPortfolio,
                WalletAction::TradeSpot,
                WalletAction::TradeMargin,
                WalletAction::SignWithdrawal,
                WalletAction::StakeYield,
            ]),
            enabled_auth_providers: HashSet::from([
                AuthProvider::EmailPassword,
                AuthProvider::Google,
                AuthProvider::Wallet,
                AuthProvider::Passkey,
            ]),
            step_up_max_age_seconds: 300,
        }
    }
}

impl BrokerageSecurityPolicy {
    /// Explicit demo policy for flows whose production MFA, trusted-device,
    /// and compliance enrollment integrations are not implemented yet.
    /// Wallet ownership, roles and action permissions are enforced separately.
    pub fn for_demo() -> Self {
        Self {
            require_email_verification: false,
            require_phone_verification: false,
            require_kyc: false,
            require_mfa_enrollment: false,
            require_terms_acceptance: false,
            require_trusted_device: false,
            require_risk_assessment: false,
            blocking_risk_tiers: HashSet::from([RiskTier::HighRisk]),
            allowed_wallet_actions: HashSet::from([
                WalletAction::ViewPortfolio,
                WalletAction::TradeSpot,
                WalletAction::TradeMargin,
                WalletAction::SignWithdrawal,
                WalletAction::StakeYield,
            ]),
            enabled_auth_providers: HashSet::from([
                AuthProvider::EmailPassword,
                AuthProvider::Google,
                AuthProvider::Wallet,
                AuthProvider::Passkey,
            ]),
            step_up_max_age_seconds: 300,
        }
    }

    /// Creates a relaxed policy suitable for testing.
    /// This bypasses all onboarding requirements while keeping the allowed actions.
    pub fn for_testing() -> Self {
        let mut policy = Self::for_demo();
        policy.blocking_risk_tiers.clear();
        policy
    }
}

#[derive(Debug, Clone)]
pub struct BrokerageAuthService {
    policy: BrokerageSecurityPolicy,
}

impl BrokerageAuthService {
    pub fn new(policy: BrokerageSecurityPolicy) -> Self {
        Self { policy }
    }

    pub fn profile_from_verified_google(
        &self,
        user_id: impl Into<String>,
        identity: &VerifiedGoogleIdentity,
    ) -> Result<IdentityProfile, AuthError> {
        if !self
            .policy
            .enabled_auth_providers
            .contains(&AuthProvider::Google)
        {
            return Err(AuthError::InvalidProvider);
        }
        Ok(IdentityProfile::from_google(user_id, identity))
    }

    pub fn profile_from_email_password(
        &self,
        user_id: impl Into<String>,
        email: impl Into<String>,
    ) -> Result<IdentityProfile, AuthError> {
        if !self
            .policy
            .enabled_auth_providers
            .contains(&AuthProvider::EmailPassword)
        {
            return Err(AuthError::InvalidProvider);
        }
        let email = email.into();
        if email.trim().is_empty() {
            return Err(AuthError::InvalidIdentityProof);
        }
        Ok(IdentityProfile::from_email_password(user_id, email))
    }

    pub fn onboarding_status(&self, profile: &IdentityProfile) -> BrokerageOnboardingStatus {
        // A hard risk block must never be masked by a missing lower-severity check.
        if self.policy.blocking_risk_tiers.contains(&profile.risk_tier) {
            return BrokerageOnboardingStatus::Blocked;
        }
        if (self.policy.require_risk_assessment
            && matches!(profile.risk_tier, RiskTier::Unassessed))
            || (self.policy.require_email_verification && !profile.email_verified)
            || (self.policy.require_phone_verification && !profile.phone_verified)
            || (self.policy.require_terms_acceptance && profile.terms_version.is_none())
            || (self.policy.require_mfa_enrollment && !profile.mfa_enrolled)
            || (self.policy.require_kyc && !profile.kyc_verified)
            || (self.policy.require_trusted_device && !profile.trusted_device)
            || profile.wallet_ids.is_empty()
        {
            return BrokerageOnboardingStatus::RequiresReview;
        }
        BrokerageOnboardingStatus::Approved
    }

    pub fn can_access_trading(&self, profile: &IdentityProfile) -> bool {
        matches!(
            self.onboarding_status(profile),
            BrokerageOnboardingStatus::Approved
        )
    }

    pub fn authorize_wallet_action(
        &self,
        principal: &AuthPrincipal,
        profile: &IdentityProfile,
        wallet: &WalletConnection,
        action: WalletAction,
        mfa_verified_at: Option<u64>,
        now: u64,
    ) -> Result<(), AuthError> {
        if principal.user_id() != profile.user_id() || wallet.owner_user_id() != principal.user_id()
        {
            return Err(AuthError::WalletNotOwned);
        }
        if !self.policy.allowed_wallet_actions.contains(&action)
            || !principal.has_permission(action)
        {
            return Err(AuthError::ForbiddenAction);
        }
        // Back-office roles never inherit customer transaction powers.
        if action.is_financial() && !principal.has_role(UserRole::Trader) {
            return Err(AuthError::ForbiddenAction);
        }
        if action.is_financial() && !self.can_access_trading(profile) {
            return Err(AuthError::OnboardingIncomplete);
        }
        if action.requires_fresh_mfa() {
            if !profile.mfa_enrolled() {
                return Err(AuthError::MfaRequired);
            }
            let verified_at = mfa_verified_at.ok_or(AuthError::MfaRequired)?;
            if verified_at > now
                || now.saturating_sub(verified_at) > self.policy.step_up_max_age_seconds
            {
                return Err(AuthError::MfaRequired);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod secure_tests {
    use super::*;

    const NOW: u64 = 1_800_000_000;

    fn service() -> BrokerageAuthService {
        BrokerageAuthService::new(BrokerageSecurityPolicy::default())
    }

    fn identity() -> VerifiedGoogleIdentity {
        VerifiedGoogleIdentity::from_verified_claims(
            "https://accounts.google.com",
            "google-subject-42",
            "trader@example.com",
            true,
        )
        .unwrap()
    }

    fn approved_profile(user_id: &str) -> IdentityProfile {
        let auth = service();
        let mut profile = auth
            .profile_from_verified_google(user_id, &identity())
            .unwrap();
        profile.apply_verified_compliance(
            VerifiedComplianceEvidence::new(true, true, true, true, RiskTier::Standard, NOW)
                .unwrap(),
        );
        profile.accept_terms("2026-08").unwrap();
        profile.record_verified_wallet("wallet-1");
        profile
    }

    fn wallet(owner: &str) -> WalletConnection {
        WalletConnection::from_verified_proof(
            "wallet-1",
            owner,
            SupportedChain::Ethereum,
            "0x1111111111111111111111111111111111111111",
            WalletProvider::MetaMask,
            NOW,
        )
        .unwrap()
    }

    fn principal(
        user_id: &str,
        role: UserRole,
        permissions: impl IntoIterator<Item = WalletAction>,
    ) -> AuthPrincipal {
        AuthPrincipal::new(
            user_id,
            HashSet::from([role]),
            permissions.into_iter().collect(),
        )
    }

    #[test]
    fn unverified_google_claims_cannot_create_identity() {
        let result = VerifiedGoogleIdentity::from_verified_claims(
            "https://accounts.google.com",
            "subject",
            "attacker@example.com",
            false,
        );
        assert_eq!(result.unwrap_err(), AuthError::UnverifiedEmail);
    }

    #[test]
    fn google_profile_starts_unassessed_and_untrusted() {
        let auth = service();
        let profile = auth
            .profile_from_verified_google("user-1", &identity())
            .unwrap();
        assert_eq!(profile.risk_tier(), RiskTier::Unassessed);
        assert_eq!(
            auth.onboarding_status(&profile),
            BrokerageOnboardingStatus::RequiresReview
        );
    }

    #[test]
    fn high_risk_is_blocked_before_incomplete_checks() {
        let auth = service();
        let mut profile = auth
            .profile_from_verified_google("user-1", &identity())
            .unwrap();
        profile.apply_verified_compliance(
            VerifiedComplianceEvidence::new(false, false, false, false, RiskTier::HighRisk, NOW)
                .unwrap(),
        );
        assert_eq!(
            auth.onboarding_status(&profile),
            BrokerageOnboardingStatus::Blocked
        );
    }

    #[test]
    fn requires_review_cannot_trade() {
        let auth = service();
        let profile = auth
            .profile_from_verified_google("user-1", &identity())
            .unwrap();
        let result = auth.authorize_wallet_action(
            &principal("user-1", UserRole::Trader, [WalletAction::TradeSpot]),
            &profile,
            &wallet("user-1"),
            WalletAction::TradeSpot,
            None,
            NOW,
        );
        assert_eq!(result.unwrap_err(), AuthError::OnboardingIncomplete);
    }

    #[test]
    fn cross_user_wallet_is_rejected() {
        let auth = service();
        let result = auth.authorize_wallet_action(
            &principal("user-1", UserRole::Trader, [WalletAction::TradeSpot]),
            &approved_profile("user-1"),
            &wallet("user-2"),
            WalletAction::TradeSpot,
            None,
            NOW,
        );
        assert_eq!(result.unwrap_err(), AuthError::WalletNotOwned);
    }

    #[test]
    fn compliance_officer_cannot_sign_withdrawals() {
        let auth = service();
        let result = auth.authorize_wallet_action(
            &principal(
                "user-1",
                UserRole::ComplianceOfficer,
                [WalletAction::SignWithdrawal],
            ),
            &approved_profile("user-1"),
            &wallet("user-1"),
            WalletAction::SignWithdrawal,
            Some(NOW),
            NOW,
        );
        assert_eq!(result.unwrap_err(), AuthError::ForbiddenAction);
    }

    #[test]
    fn sensitive_action_requires_fresh_mfa() {
        let auth = service();
        let principal = principal("user-1", UserRole::Trader, [WalletAction::SignWithdrawal]);
        let profile = approved_profile("user-1");
        let wallet = wallet("user-1");
        assert_eq!(
            auth.authorize_wallet_action(
                &principal,
                &profile,
                &wallet,
                WalletAction::SignWithdrawal,
                Some(NOW - 301),
                NOW,
            )
            .unwrap_err(),
            AuthError::MfaRequired
        );
        assert!(auth
            .authorize_wallet_action(
                &principal,
                &profile,
                &wallet,
                WalletAction::SignWithdrawal,
                Some(NOW - 30),
                NOW,
            )
            .is_ok());
    }

    #[test]
    fn reconstituted_profile_matches_original_field_for_field() {
        let original = approved_profile("user-1");
        let rehydrated = IdentityProfile::reconstitute(
            original.user_id().to_string(),
            original.email().to_string(),
            original.email_verified(),
            original.phone_verified(),
            original.kyc_verified(),
            original.terms_version().map(str::to_string),
            original.mfa_enrolled(),
            original.trusted_device(),
            original.risk_tier(),
            original.compliance_checked_at(),
            original.wallet_ids().to_vec(),
            original.auth_provider(),
        );
        assert_eq!(original, rehydrated);
    }

    #[test]
    fn reconstituted_profile_is_evaluated_identically_to_a_fresh_one() {
        let auth = service();
        let original = approved_profile("user-1");
        let rehydrated = IdentityProfile::reconstitute(
            original.user_id().to_string(),
            original.email().to_string(),
            original.email_verified(),
            original.phone_verified(),
            original.kyc_verified(),
            original.terms_version().map(str::to_string),
            original.mfa_enrolled(),
            original.trusted_device(),
            original.risk_tier(),
            original.compliance_checked_at(),
            original.wallet_ids().to_vec(),
            original.auth_provider(),
        );

        assert_eq!(
            auth.onboarding_status(&original),
            auth.onboarding_status(&rehydrated)
        );

        let principal = principal("user-1", UserRole::Trader, [WalletAction::TradeSpot]);
        let wallet = wallet("user-1");
        assert_eq!(
            auth.authorize_wallet_action(
                &principal,
                &original,
                &wallet,
                WalletAction::TradeSpot,
                None,
                NOW,
            ),
            auth.authorize_wallet_action(
                &principal,
                &rehydrated,
                &wallet,
                WalletAction::TradeSpot,
                None,
                NOW,
            )
        );
    }
}
