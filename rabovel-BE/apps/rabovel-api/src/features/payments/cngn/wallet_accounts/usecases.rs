use crate::app::constants::SESSION_RATE_LIMIT;
use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::wallets::queries::usecases::investor_solana_wallet;
use crate::features::payments::cngn::models::WalletPaymentAssetStatus;
use identity::AuthError;

pub(crate) async fn investor_cngn_account(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<WalletPaymentAssetStatus, ApplicationError> {
    let wallet = investor_solana_wallet(state, &user.user_id).await?;
    state
        .payment_assets()
        .cngn_wallet_status(wallet.address())
        .await
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })
}

pub(crate) async fn create_investor_cngn_account(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<WalletPaymentAssetStatus, ApplicationError> {
    state
        .rate_limiter()
        .check(
            &format!("session:{}", user.user_id),
            SESSION_RATE_LIMIT.0,
            SESSION_RATE_LIMIT.1,
        )
        .await
        .map_err(ApplicationError::from)?;
    let wallet = investor_solana_wallet(state, &user.user_id).await?;
    state
        .payment_assets()
        .ensure_cngn_wallet_account(wallet.address())
        .await
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::payments::cngn::ports::PaymentProvider
    + crate::features::identity::access::ports::RateLimit
    + crate::features::identity::wallets::queries::ports::WalletLookup
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::payments::cngn::ports::PaymentProvider
            + crate::features::identity::access::ports::RateLimit
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
