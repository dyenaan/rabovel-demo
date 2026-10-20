use crate::*;
use std::collections::HashSet;

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
        VerifiedComplianceEvidence::new(true, true, true, true, RiskTier::Standard, NOW).unwrap(),
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
