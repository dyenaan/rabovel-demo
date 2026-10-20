use crate::app::errors::ApplicationError;
use crate::features::compliance::onboarding::dto::OnboardingStatusResponse;
use crate::features::compliance::onboarding::dto::TermsAcceptanceRequest;
use crate::features::identity::access::actor::AuthenticatedUser;
use events::schema::onboarding::OnboardingStatus as EventOnboardingStatus;
use events::schema::onboarding::OnboardingStatusChanged;
use identity::AuthError;
use identity::BrokerageOnboardingStatus;

use crate::app::support::unix_now;

pub(crate) async fn onboarding_status(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<OnboardingStatusResponse, ApplicationError> {
    let record = state
        .account_queries()
        .get_user(&user.user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    Ok(OnboardingStatusResponse {
        status: state.auth_service().onboarding_status(&record.profile),
        profile: record.profile,
    })
}

pub(crate) async fn accept_terms(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: TermsAcceptanceRequest,
) -> Result<OnboardingStatusResponse, ApplicationError> {
    let mut record = state
        .account_queries()
        .get_user(&user.user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    let before_status = state.auth_service().onboarding_status(&record.profile);
    record
        .profile
        .accept_terms(request.version)
        .map_err(ApplicationError::from)?;
    state
        .profile_updates()
        .save_profile(record.profile.clone())
        .await
        .map_err(ApplicationError::from)?;
    let after_status = state.auth_service().onboarding_status(&record.profile);
    publish_onboarding_change(state, &user.user_id, before_status, after_status).await;
    Ok(OnboardingStatusResponse {
        status: after_status,
        profile: record.profile,
    })
}

pub(crate) async fn publish_onboarding_change(
    state: &impl Ports,
    user_id: &str,
    before: BrokerageOnboardingStatus,
    after: BrokerageOnboardingStatus,
) {
    if before == after {
        return;
    }
    let _ = events::publish(
        state.event_producer().as_ref(),
        OnboardingStatusChanged {
            user_id: user_id.to_string(),
            previous_status: to_event_status(before),
            new_status: to_event_status(after),
            occurred_at: unix_now(),
        },
        "order-gateway",
    )
    .await;
}

pub(crate) fn to_event_status(status: BrokerageOnboardingStatus) -> EventOnboardingStatus {
    match status {
        BrokerageOnboardingStatus::Approved => EventOnboardingStatus::Approved,
        BrokerageOnboardingStatus::RequiresReview => EventOnboardingStatus::RequiresReview,
        BrokerageOnboardingStatus::Blocked => EventOnboardingStatus::Blocked,
    }
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::identity::access::ports::AccessPolicy
    + crate::features::compliance::onboarding::ports::EventPublisher
    + crate::features::identity::accounts::ports::ProfileUpdates
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::identity::access::ports::AccessPolicy
            + crate::features::compliance::onboarding::ports::EventPublisher
            + crate::features::identity::accounts::ports::ProfileUpdates
            + ?Sized,
    > Ports for T
{
}
