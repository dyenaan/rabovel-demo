#![cfg(debug_assertions)]

use crate::app::errors::ApplicationError;
use crate::features::compliance::kyc::cases::dto::SandboxAdvanceRequest;
use crate::features::compliance::kyc::verdicts::usecases::kyc_webhook;
use identity::AuthError;

#[cfg(debug_assertions)]
pub(crate) async fn kyc_sandbox_advance(
    state: &impl Ports,
    request: SandboxAdvanceRequest,
) -> Result<(), ApplicationError> {
    let sandbox = state
        .sandbox_kyc()
        .clone()
        .ok_or_else(|| ApplicationError::from(AuthError::InvalidProvider))?;
    let (body, signature) = sandbox
        .force_webhook_payload(&request.case_id)
        .map_err(ApplicationError::from)?;
    kyc_webhook(state, &signature, &body).await
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::identity::access::ports::AccessPolicy
    + crate::features::compliance::kyc::cases::ports::CaseStore
    + crate::features::compliance::onboarding::ports::EventPublisher
    + crate::features::compliance::kyc::cases::ports::KycProvider
    + crate::features::identity::accounts::ports::ProfileUpdates
    + crate::features::compliance::kyc::sandbox::ports::SandboxKyc
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::identity::access::ports::AccessPolicy
            + crate::features::compliance::kyc::cases::ports::CaseStore
            + crate::features::compliance::onboarding::ports::EventPublisher
            + crate::features::compliance::kyc::cases::ports::KycProvider
            + crate::features::identity::accounts::ports::ProfileUpdates
            + crate::features::compliance::kyc::sandbox::ports::SandboxKyc
            + ?Sized,
    > Ports for T
{
}
