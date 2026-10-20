use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::organizations::usecases::require_approved_issuer;
use crate::features::payments::cngn::models::PaymentAssetStatus;
use identity::AuthError;

pub(crate) async fn cngn_payment_asset(
    state: &impl Ports,
    _user: AuthenticatedUser,
) -> PaymentAssetStatus {
    state.payment_assets().cngn_status().await
}

pub(crate) async fn create_broker_cngn_account(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<PaymentAssetStatus, ApplicationError> {
    require_approved_issuer(state, &user.user_id).await?;
    state
        .payment_assets()
        .ensure_cngn_broker_account()
        .await
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::organizations::ports::OrganizationStore
    + crate::features::payments::cngn::ports::PaymentProvider
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::organizations::ports::OrganizationStore
            + crate::features::payments::cngn::ports::PaymentProvider
            + ?Sized,
    > Ports for T
{
}
