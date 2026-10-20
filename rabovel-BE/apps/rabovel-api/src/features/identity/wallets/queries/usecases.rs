use crate::app::errors::ApplicationError;
use crate::features::identity::access::actor::AuthenticatedUser;
use identity::AuthError;
use identity::SupportedChain;
use identity::UserRole;
use identity::WalletConnection;

pub(crate) async fn list_wallets(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<Vec<WalletConnection>, ApplicationError> {
    state
        .wallet_queries()
        .list_wallets(&user.user_id)
        .await
        .map_err(ApplicationError::from)
}

pub(crate) async fn investor_solana_wallet(
    state: &impl Ports,
    user_id: &str,
) -> Result<WalletConnection, ApplicationError> {
    let user = state
        .account_queries()
        .get_user(user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    if !user.principal.has_role(UserRole::Trader) {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }
    state
        .wallet_queries()
        .list_wallets(user_id)
        .await
        .map_err(ApplicationError::from)?
        .into_iter()
        .find(|wallet| wallet.chain() == SupportedChain::Solana)
        .ok_or_else(|| ApplicationError::from(AuthError::MissingWallet))
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::identity::wallets::queries::ports::WalletLookup
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
