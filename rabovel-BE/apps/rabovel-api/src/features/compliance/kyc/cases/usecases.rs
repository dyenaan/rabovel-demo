use crate::app::constants::SESSION_RATE_LIMIT;
use crate::app::errors::ApplicationError;
use crate::features::compliance::kyc::cases::dto::KycCaseRequest;
use crate::features::compliance::kyc::cases::dto::KycCaseResponse;
use crate::features::identity::access::actor::AuthenticatedUser;
use events::schema::kyc::KycCaseSubmitted;
use identity::AuthError;
use kyc::cases::models::KycCaseRecord;
use kyc::cases::models::KycCaseStatus;
use kyc::KycSubmission;

use crate::app::support::random_hex;

pub(crate) async fn submit_kyc_case(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: KycCaseRequest,
) -> Result<KycCaseResponse, ApplicationError> {
    state
        .rate_limiter()
        .check(
            &format!("session:{}", user.user_id),
            SESSION_RATE_LIMIT.0,
            SESSION_RATE_LIMIT.1,
        )
        .await
        .map_err(ApplicationError::from)?;
    // Confirms the caller is a real session-bound user before we ever talk
    // to the KYC provider on their behalf.
    state
        .account_queries()
        .get_user(&user.user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;

    let submission = KycSubmission {
        user_id: user.user_id.clone(),
        full_name: request.full_name,
        date_of_birth: request.date_of_birth,
        country: request.country,
        document_type: request.document_type,
        document_reference: request.document_reference,
        request_id: random_hex(16).map_err(ApplicationError::from)?,
    };
    let handle = state
        .kyc_provider()
        .submit_case(submission)
        .await
        .map_err(ApplicationError::from)?;
    state
        .cases()
        .create_kyc_case(KycCaseRecord {
            case_id: handle.case_id.clone(),
            user_id: user.user_id.clone(),
            status: KycCaseStatus::Submitted,
            risk_tier: None,
            submitted_at: handle.submitted_at,
            finalized_at: None,
        })
        .await
        .map_err(ApplicationError::from)?;

    let _ = events::publish(
        state.event_producer().as_ref(),
        KycCaseSubmitted {
            case_id: handle.case_id.clone(),
            user_id: user.user_id,
            submitted_at: handle.submitted_at,
        },
        "order-gateway",
    )
    .await;

    Ok(KycCaseResponse {
        case_id: handle.case_id,
        submitted_at: handle.submitted_at,
    })
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::compliance::kyc::cases::ports::CaseStore
    + crate::features::compliance::onboarding::ports::EventPublisher
    + crate::features::compliance::kyc::cases::ports::KycProvider
    + crate::features::identity::access::ports::RateLimit
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::compliance::kyc::cases::ports::CaseStore
            + crate::features::compliance::onboarding::ports::EventPublisher
            + crate::features::compliance::kyc::cases::ports::KycProvider
            + crate::features::identity::access::ports::RateLimit
            + ?Sized,
    > Ports for T
{
}
