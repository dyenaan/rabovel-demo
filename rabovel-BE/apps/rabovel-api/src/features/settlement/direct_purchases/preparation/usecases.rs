use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::wallets::queries::usecases::investor_solana_wallet;
use crate::features::settlement::direct_purchases::preparation::dto::PreparePurchaseRequest;
use crate::features::settlement::direct_purchases::preparation::dto::PreparedPurchaseResponse;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteRequest;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteSide;
use crate::features::settlement::direct_purchases::quotes::usecases::create_investor_quote;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use identity::AuthError;
use settlement_orchestrator::direct_purchases::preparation::models::PurchaseTerms;

pub(crate) async fn prepare_investor_purchase(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: PreparePurchaseRequest,
) -> Result<PreparedPurchaseResponse, ApplicationError> {
    let quote = create_investor_quote(
        state,
        &user,
        QuoteRequest {
            asset_id: request.asset_id.clone(),
            side: QuoteSide::Buy,
            quantity: request.quantity,
        },
    )
    .await?;
    let wallet = investor_solana_wallet(state, &user.user_id).await?;
    let asset = state
        .asset_queries()
        .get_asset_draft(&request.asset_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::NotFound,
            error: AuthError::PolicyViolation("asset not found".into()),
        })?;
    if asset.draft.listing_status != "live" {
        return Err(ApplicationError {
            kind: FailureKind::Conflict,
            error: AuthError::PolicyViolation("asset listing is not live".into()),
        });
    }
    let operation = state
        .setup_queries()
        .get_asset_setup_operation(&request.asset_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::Conflict,
            error: AuthError::PolicyViolation("asset setup is unavailable".into()),
        })?;
    let payment = state.payment_assets().cngn_status().await;
    if !payment.ready {
        return Err(ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(
                payment
                    .error
                    .unwrap_or_else(|| "broker cNGN account is not ready".into()),
            ),
        });
    }
    let token_program = payment
        .token_program
        .as_deref()
        .and_then(|value| match value {
            "spl-token" => Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            "spl-token-2022" => Some("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
            _ => None,
        })
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation("unsupported cNGN token program".into()),
        })?;
    let quantity = quote.quantity.parse::<u64>().map_err(|_| {
        ApplicationError::from(AuthError::PolicyViolation("invalid quote quantity".into()))
    })?;
    let total = quote.total_payment.parse::<u64>().map_err(|_| {
        ApplicationError::from(AuthError::PolicyViolation("invalid quote total".into()))
    })?;
    let transaction = state
        .purchase_preparation()
        .prepare_purchase(
            &asset,
            &operation,
            PurchaseTerms {
                investor_wallet: wallet.address(),
                investor_payment_account: &quote.payment_token_account,
                payment_mint: &quote.payment_mint,
                payment_token_program: token_program,
                payment_decimals: payment.decimals.unwrap_or(6),
                broker_payment_account: payment.broker_token_account.as_deref().ok_or_else(
                    || ApplicationError {
                        kind: FailureKind::Unavailable,
                        error: AuthError::PolicyViolation(
                            "broker cNGN account is unavailable".into(),
                        ),
                    },
                )?,
                equity_quantity: quantity,
                payment_amount: total,
            },
        )
        .await
        .map_err(|_| ApplicationError::from(AuthError::ServiceUnavailable))?;
    Ok(PreparedPurchaseResponse {
        transaction_base64: BASE64.encode(transaction),
        quote,
    })
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::assets::listings::ports::PublishedAssets
    + crate::features::identity::access::ports::AccessPolicy
    + crate::features::issuer::assets::issuance::inventory::ports::Inventory
    + crate::features::payments::cngn::ports::PaymentProvider
    + crate::features::issuer::assets::issuance::inventory::ports::Positions
    + crate::features::market_data::prices::ports::ReferencePrices
    + crate::features::settlement::direct_purchases::preparation::ports::Preparation
    + crate::features::settlement::direct_purchases::quotes::ports::QuotePolicy
    + crate::features::identity::access::ports::RateLimit
    + crate::features::issuer::assets::issuance::setup::ports::SetupLookup
    + crate::features::identity::wallets::queries::ports::WalletLookup
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::assets::listings::ports::PublishedAssets
            + crate::features::identity::access::ports::AccessPolicy
            + crate::features::issuer::assets::issuance::inventory::ports::Inventory
            + crate::features::payments::cngn::ports::PaymentProvider
            + crate::features::issuer::assets::issuance::inventory::ports::Positions
            + crate::features::market_data::prices::ports::ReferencePrices
            + crate::features::settlement::direct_purchases::preparation::ports::Preparation
            + crate::features::settlement::direct_purchases::quotes::ports::QuotePolicy
            + crate::features::identity::access::ports::RateLimit
            + crate::features::issuer::assets::issuance::setup::ports::SetupLookup
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
