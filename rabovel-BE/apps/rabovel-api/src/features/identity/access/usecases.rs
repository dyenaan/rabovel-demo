use crate::app::constants::SESSION_RATE_LIMIT;
use crate::app::errors::ApplicationError;
use crate::app::support::unix_now;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::access::dto::WalletAuthorizationRequest;
use crate::features::identity::access::dto::WalletAuthorizationResponse;
use identity::AuthError;
use identity::UserRecord;
use identity::WalletAction;

pub(crate) async fn authorize_wallet(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: WalletAuthorizationRequest,
) -> Result<WalletAuthorizationResponse, ApplicationError> {
    state
        .rate_limiter()
        .check(
            &format!("session:{}", user.user_id),
            SESSION_RATE_LIMIT.0,
            SESSION_RATE_LIMIT.1,
        )
        .await
        .map_err(ApplicationError::from)?;
    let record = state
        .account_queries()
        .get_user(&user.user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    let wallet = state
        .wallet_queries()
        .get_wallet(&request.wallet_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::MissingWallet))?;
    state
        .auth_service()
        .authorize_wallet_action(
            &record.principal,
            &record.profile,
            &wallet,
            request.action,
            user.mfa_verified_at,
            unix_now(),
        )
        .map_err(ApplicationError::from)?;
    Ok(WalletAuthorizationResponse { authorized: true })
}

/// Reuses the existing wallet-authorization gate (ownership + role +
/// permission + onboarding + MFA freshness) for portfolio/trading
/// actions by authorizing against the user's first linked wallet. These
/// routes are not inherently wallet-specific, but this keeps a single,
/// already-audited authorization path in front of every financial
/// action instead of a second parallel one.
pub(crate) async fn authorize_trading_action(
    state: &impl Ports,
    user: &AuthenticatedUser,
    action: WalletAction,
) -> Result<UserRecord, ApplicationError> {
    crate::features::identity::access::policy::authorize_trading_action(
        state.account_queries().as_ref(),
        state.wallet_queries().as_ref(),
        state.auth_service().as_ref(),
        &user.user_id,
        user.mfa_verified_at,
        action,
    )
    .await
    .map_err(ApplicationError::from)
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::identity::access::ports::AccessPolicy
    + crate::features::identity::access::ports::RateLimit
    + crate::features::identity::wallets::queries::ports::WalletLookup
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::identity::access::ports::AccessPolicy
            + crate::features::identity::access::ports::RateLimit
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
