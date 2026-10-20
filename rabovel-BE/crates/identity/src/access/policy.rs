use crate::*;
use std::collections::HashSet;

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
