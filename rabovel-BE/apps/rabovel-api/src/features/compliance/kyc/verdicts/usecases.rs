use crate::app::errors::ApplicationError;
use events::schema::kyc::KycRiskTier;
use events::schema::kyc::KycVerdictReceived;
use identity::AuthError;
use identity::RiskTier;
use kyc::cases::models::KycCaseStatus;
use kyc::KycError;

use crate::features::compliance::onboarding::usecases::publish_onboarding_change;

pub(crate) async fn kyc_webhook(
    state: &impl Ports,
    signature: &str,
    body: &[u8],
) -> Result<(), ApplicationError> {
    let verdict = state
        .kyc_provider()
        .verify_webhook(body, signature)
        .map_err(ApplicationError::from)?;

    let case = state
        .cases()
        .get_kyc_case(&verdict.case_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(KycError::CaseNotFound))?;
    if case.user_id != verdict.user_id {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "kyc verdict user mismatch".to_string(),
        )));
    }

    let mut record = state
        .account_queries()
        .get_user(&verdict.user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    let before_status = state.auth_service().onboarding_status(&record.profile);

    let evidence = verdict
        .clone()
        .into_compliance_evidence(
            record.profile.mfa_enrolled(),
            record.profile.trusted_device(),
        )
        .map_err(ApplicationError::from)?;
    record.profile.apply_verified_compliance(evidence);
    state
        .profile_updates()
        .save_profile(record.profile.clone())
        .await
        .map_err(ApplicationError::from)?;

    let case_status = if verdict.kyc_verified {
        KycCaseStatus::Approved
    } else {
        KycCaseStatus::Rejected
    };
    state
        .cases()
        .finalize_kyc_case(
            &verdict.case_id,
            case_status,
            Some(verdict.risk_tier),
            verdict.checked_at,
        )
        .await
        .map_err(ApplicationError::from)?;

    let after_status = state.auth_service().onboarding_status(&record.profile);

    let _ = events::publish(
        state.event_producer().as_ref(),
        KycVerdictReceived {
            case_id: verdict.case_id.clone(),
            user_id: verdict.user_id.clone(),
            kyc_verified: verdict.kyc_verified,
            risk_tier: to_event_risk_tier(verdict.risk_tier),
            checked_at: verdict.checked_at,
        },
        "order-gateway",
    )
    .await;
    publish_onboarding_change(state, &verdict.user_id, before_status, after_status).await;

    Ok(())
}

pub(crate) fn to_event_risk_tier(tier: RiskTier) -> KycRiskTier {
    match tier {
        RiskTier::Standard => KycRiskTier::Standard,
        RiskTier::Enhanced => KycRiskTier::Enhanced,
        // `Unassessed` should never reach here: `VerifiedComplianceEvidence::new`
        // (called by `VerifiedKycVerdict::into_compliance_evidence`) rejects it
        // upstream. Mapped conservatively rather than panicking.
        RiskTier::HighRisk | RiskTier::Unassessed => KycRiskTier::HighRisk,
    }
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::identity::access::ports::AccessPolicy
    + crate::features::compliance::kyc::cases::ports::CaseStore
    + crate::features::compliance::onboarding::ports::EventPublisher
    + crate::features::compliance::kyc::cases::ports::KycProvider
    + crate::features::identity::accounts::ports::ProfileUpdates
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::identity::access::ports::AccessPolicy
            + crate::features::compliance::kyc::cases::ports::CaseStore
            + crate::features::compliance::onboarding::ports::EventPublisher
            + crate::features::compliance::kyc::cases::ports::KycProvider
            + crate::features::identity::accounts::ports::ProfileUpdates
            + ?Sized,
    > Ports for T
{
}
