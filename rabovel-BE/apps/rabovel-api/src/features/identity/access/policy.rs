use crate::app::support::unix_now;
use identity::AuthError;
use identity::BrokerageAuthService;
use identity::UserRecord;
use identity::WalletAction;

/// Shared with the remaining legacy handlers until their feature extraction.
/// Policy remains centralized: current profile, first wallet, ownership, role,
/// permission, onboarding and MFA are checked by the existing domain service.
pub(crate) async fn authorize_trading_action(
    accounts: &dyn identity::accounts::ports::AccountQueries,
    wallets: &dyn identity::wallets::ports::WalletQueries,
    auth_service: &BrokerageAuthService,
    user_id: &str,
    mfa_verified_at: Option<u64>,
    action: WalletAction,
) -> Result<UserRecord, AuthError> {
    let record = accounts
        .get_user(user_id)
        .await?
        .ok_or(AuthError::Unauthorized)?;
    let wallet_id = record
        .profile
        .wallet_ids()
        .first()
        .ok_or(AuthError::MissingWallet)?;
    let wallet = wallets
        .get_wallet(wallet_id)
        .await?
        .ok_or(AuthError::MissingWallet)?;
    auth_service.authorize_wallet_action(
        &record.principal,
        &record.profile,
        &wallet,
        action,
        mfa_verified_at,
        unix_now(),
    )?;
    Ok(record)
}
