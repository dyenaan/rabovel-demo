#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    ServiceUnavailable,
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
            Self::ServiceUnavailable => "service_unavailable",
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
            Self::ServiceUnavailable => write!(f, "the service is temporarily unavailable"),
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
