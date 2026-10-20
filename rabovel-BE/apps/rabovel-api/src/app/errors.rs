use identity::AuthError;
use kyc::KycError;

#[derive(Debug)]
pub(crate) struct ApplicationError {
    pub(crate) kind: FailureKind,
    pub(crate) error: AuthError,
}
impl From<AuthError> for ApplicationError {
    fn from(error: AuthError) -> Self {
        let kind = match error {
            AuthError::Unauthorized | AuthError::SessionExpired => FailureKind::Unauthenticated,
            AuthError::ForbiddenAction
            | AuthError::WalletNotOwned
            | AuthError::OnboardingIncomplete
            | AuthError::MfaRequired => FailureKind::Forbidden,
            AuthError::ReplayDetected => FailureKind::Conflict,
            AuthError::InvalidProvider | AuthError::ServiceUnavailable => FailureKind::Unavailable,
            AuthError::InvalidIdentityProof
            | AuthError::UnverifiedEmail
            | AuthError::InvalidWalletProof => FailureKind::Unauthenticated,
            AuthError::ChallengeExpired => FailureKind::Expired,
            AuthError::MissingWallet => FailureKind::NotFound,
            AuthError::RateLimited => FailureKind::RateLimited,
            AuthError::PolicyViolation(_) => FailureKind::InvalidInput,
        };
        Self { kind, error }
    }
}
impl From<KycError> for ApplicationError {
    fn from(error: KycError) -> Self {
        let kind = match error {
            KycError::CaseNotFound => FailureKind::NotFound,
            KycError::InvalidSignature | KycError::MalformedPayload => FailureKind::InvalidInput,
            KycError::Unavailable(_) => FailureKind::Unavailable,
        };
        Self {
            kind,
            error: AuthError::PolicyViolation(error.to_string()),
        }
    }
}
impl From<portfolio_trading::TradingError> for ApplicationError {
    fn from(error: portfolio_trading::TradingError) -> Self {
        let kind = match error {
            portfolio_trading::TradingError::OrderNotFound => FailureKind::NotFound,
            portfolio_trading::TradingError::InvalidQuantity(_)
            | portfolio_trading::TradingError::InvalidPrice(_)
            | portfolio_trading::TradingError::InvalidOrder(_)
            | portfolio_trading::TradingError::UnknownSymbol(_)
            | portfolio_trading::TradingError::OrderTooLarge(_) => FailureKind::InvalidInput,
            portfolio_trading::TradingError::Unavailable(_) => FailureKind::Unavailable,
        };
        Self {
            kind,
            error: AuthError::PolicyViolation(error.to_string()),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FailureKind {
    Unauthenticated,
    Forbidden,
    Conflict,
    Unavailable,
    Expired,
    NotFound,
    RateLimited,
    InvalidInput,
}
impl FailureKind {
    pub(crate) fn is_server_error(self) -> bool {
        matches!(self, Self::Unavailable)
    }
}
