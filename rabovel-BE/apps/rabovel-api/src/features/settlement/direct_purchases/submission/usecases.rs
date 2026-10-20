use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::wallets::queries::usecases::investor_solana_wallet;
use crate::features::settlement::direct_purchases::submission::dto::PurchaseSettlementResponse;
use crate::features::settlement::direct_purchases::submission::dto::SubmitPurchaseRequest;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use identity::AuthError;
use identity::WalletAction;

pub(crate) async fn submit_investor_purchase(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: SubmitPurchaseRequest,
) -> Result<PurchaseSettlementResponse, ApplicationError> {
    crate::features::identity::access::usecases::authorize_trading_action(
        state,
        &user,
        WalletAction::TradeSpot,
    )
    .await?;
    let wallet = investor_solana_wallet(state, &user.user_id).await?;
    let transaction = BASE64.decode(request.transaction_base64).map_err(|_| {
        ApplicationError::from(AuthError::PolicyViolation(
            "signed purchase transaction is not valid base64".into(),
        ))
    })?;
    let signature = state
        .purchase_submission()
        .submit_purchase(wallet.address(), &transaction)
        .await
        .map_err(|error| ApplicationError {
            kind: if error.reconciliation_required {
                FailureKind::Conflict
            } else {
                FailureKind::Unavailable
            },
            error: AuthError::PolicyViolation(
                "The chain operation could not be completed. Check its status before retrying."
                    .into(),
            ),
        })?;
    Ok(PurchaseSettlementResponse {
        signature,
        status: "confirmed".into(),
    })
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::identity::access::ports::AccessPolicy
    + crate::features::settlement::direct_purchases::submission::ports::Submission
    + crate::features::identity::access::ports::RateLimit
    + crate::features::identity::wallets::queries::ports::WalletLookup
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::identity::access::ports::AccessPolicy
            + crate::features::settlement::direct_purchases::submission::ports::Submission
            + crate::features::identity::access::ports::RateLimit
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
