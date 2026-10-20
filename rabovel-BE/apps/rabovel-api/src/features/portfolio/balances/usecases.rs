use crate::app::errors::ApplicationError;
use crate::features::identity::access::actor::AuthenticatedUser;
use identity::WalletAction;
use portfolio_trading::PortfolioSnapshot;

pub(crate) async fn portfolio(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<PortfolioSnapshot, ApplicationError> {
    crate::features::identity::access::usecases::authorize_trading_action(
        state,
        &user,
        WalletAction::ViewPortfolio,
    )
    .await?;
    let snapshot = state
        .balance_queries()
        .get_portfolio(&user.user_id)
        .await
        .map_err(ApplicationError::from)?;
    Ok(snapshot)
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::identity::access::ports::AccessPolicy
    + crate::features::portfolio::balances::ports::Balances
    + crate::features::identity::access::ports::RateLimit
    + crate::features::identity::wallets::queries::ports::WalletLookup
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::identity::access::ports::AccessPolicy
            + crate::features::portfolio::balances::ports::Balances
            + crate::features::identity::access::ports::RateLimit
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
