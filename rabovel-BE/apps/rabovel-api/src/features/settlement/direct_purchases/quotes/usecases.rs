use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::wallets::queries::usecases::investor_solana_wallet;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteRequest;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteResponse;
use crate::features::settlement::direct_purchases::quotes::dto::QuoteSide;
use identity::AuthError;
use identity::WalletAction;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

pub(crate) async fn investor_quote(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: QuoteRequest,
) -> Result<QuoteResponse, ApplicationError> {
    create_investor_quote(state, &user, request).await
}

pub(crate) async fn create_investor_quote(
    state: &impl Ports,
    user: &AuthenticatedUser,
    request: QuoteRequest,
) -> Result<QuoteResponse, ApplicationError> {
    crate::features::identity::access::usecases::authorize_trading_action(
        state,
        user,
        WalletAction::TradeSpot,
    )
    .await?;

    let config = state.quote_config();
    let fee_bps = config.fee_bps;
    let quote_expiry_seconds = config.expiry_seconds;

    // Validate quantity
    let quantity: u64 = request.quantity.parse().map_err(|_| ApplicationError {
        kind: FailureKind::InvalidInput,
        error: AuthError::PolicyViolation("invalid quantity".into()),
    })?;
    if quantity == 0 {
        return Err(ApplicationError {
            kind: FailureKind::InvalidInput,
            error: AuthError::PolicyViolation("quantity must be greater than zero".into()),
        });
    }

    // Get investor's Solana wallet
    let wallet = investor_solana_wallet(state, &user.user_id).await?;

    // Get payment asset status (CNGN for now)
    let cngn = state
        .payment_assets()
        .cngn_wallet_status(wallet.address())
        .await
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })?;

    // Find the asset
    let candidates = state
        .asset_queries()
        .list_minted_assets()
        .await
        .map_err(ApplicationError::from)?;
    let mut asset_record = None;
    for asset in candidates {
        if asset.asset_id == request.asset_id {
            asset_record = Some(asset);
            break;
        }
    }
    let asset = asset_record.ok_or_else(|| ApplicationError {
        kind: FailureKind::NotFound,
        error: AuthError::PolicyViolation("asset not found".into()),
    })?;
    let market_price = state
        .price_feed()
        .latest(&asset.draft.ticker, cngn.decimals.unwrap_or(6))
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })?;
    let buy_price = market_price.price_base_units;

    // Get setup operation
    let operation = state
        .setup_queries()
        .get_asset_setup_operation(&asset.asset_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::NotFound,
            error: AuthError::PolicyViolation("asset setup operation not found".into()),
        })?;

    if operation.status != "confirmed" {
        return Err(ApplicationError {
            kind: FailureKind::InvalidInput,
            error: AuthError::PolicyViolation("asset setup is not confirmed".into()),
        });
    }

    // Get inventory
    let inventory = state
        .inventory()
        .initial_inventory(&asset, &operation, false)
        .await
        .map_err(|_| ApplicationError::from(AuthError::ServiceUnavailable))?;

    if !inventory.issuance_complete {
        return Err(ApplicationError {
            kind: FailureKind::InvalidInput,
            error: AuthError::PolicyViolation("asset inventory issuance is not complete".into()),
        });
    }

    // Get investor position
    let position = state
        .positions()
        .investor_position(&operation, wallet.address())
        .await
        .map_err(|_| ApplicationError::from(AuthError::ServiceUnavailable))?;

    // Validate based on side
    let (price_per_unit, payment_mint, payment_token_account, equity_token_account) = match request
        .side
    {
        QuoteSide::Buy => {
            // Check available inventory
            let available_inventory: u64 =
                inventory
                    .inventory_balance
                    .parse()
                    .map_err(|_| ApplicationError {
                        kind: FailureKind::Unavailable,
                        error: AuthError::PolicyViolation("invalid inventory balance".into()),
                    })?;
            if quantity > available_inventory {
                return Err(ApplicationError {
                    kind: FailureKind::InvalidInput,
                    error: AuthError::PolicyViolation("insufficient inventory available".into()),
                });
            }
            // Check payment balance
            let payment_balance: u64 = cngn
                .balance_base_units
                .as_deref()
                .unwrap_or("0")
                .parse()
                .map_err(|_| ApplicationError {
                    kind: FailureKind::Unavailable,
                    error: AuthError::PolicyViolation("invalid payment balance".into()),
                })?;
            let total_payment =
                settlement_orchestrator::direct_purchases::quotes::models::buy_amounts(
                    quantity, buy_price, fee_bps,
                )
                .map_err(|message| ApplicationError {
                    kind: FailureKind::Unavailable,
                    error: AuthError::PolicyViolation(message.into()),
                })?
                .total;
            if total_payment > payment_balance {
                return Err(ApplicationError {
                    kind: FailureKind::InvalidInput,
                    error: AuthError::PolicyViolation("insufficient payment balance".into()),
                });
            }
            (
                buy_price.to_string(),
                cngn.mint_address.clone(),
                cngn.token_account.clone().ok_or_else(|| ApplicationError {
                    kind: FailureKind::InvalidInput,
                    error: AuthError::PolicyViolation(
                        "investor payment token account not found".into(),
                    ),
                })?,
                inventory.token_account.clone(),
            )
        }
        QuoteSide::Sell => {
            // Check investor equity balance
            let investor_balance: u64 = position.balance.parse().map_err(|_| ApplicationError {
                kind: FailureKind::Unavailable,
                error: AuthError::PolicyViolation("invalid investor equity balance".into()),
            })?;
            if quantity > investor_balance {
                return Err(ApplicationError {
                    kind: FailureKind::InvalidInput,
                    error: AuthError::PolicyViolation("insufficient equity balance".into()),
                });
            }
            // For sell, use a demo bid price (e.g., 90% of buy price)
            let bid_price = buy_price * 90 / 100;
            let payment_account = cngn.token_account.clone().ok_or_else(|| ApplicationError {
                kind: FailureKind::InvalidInput,
                error: AuthError::PolicyViolation(
                    "investor payment token account not found".into(),
                ),
            })?;
            if !position.account_ready {
                return Err(ApplicationError {
                    kind: FailureKind::InvalidInput,
                    error: AuthError::PolicyViolation(
                        "investor equity token account is not ready".into(),
                    ),
                });
            }
            (
                bid_price.to_string(),
                cngn.mint_address.clone(),
                payment_account,
                position.token_account.clone(),
            )
        }
    };

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expires_at = created_at + quote_expiry_seconds;

    let quote_id = format!("quote_{}", Uuid::new_v4().simple());

    // Calculate fee and total for response
    let price: u64 = price_per_unit.parse().unwrap_or(0);
    let total_cost = quantity.checked_mul(price).unwrap_or(0);
    let fee = total_cost.checked_mul(fee_bps as u64).unwrap_or(0) / 10000;
    let total_payment = total_cost.checked_add(fee).unwrap_or(0);

    Ok(QuoteResponse {
        quote_id,
        asset_id: request.asset_id,
        side: request.side,
        quantity: request.quantity,
        price_per_unit,
        fee_bps,
        fee_amount: fee.to_string(),
        total_payment: total_payment.to_string(),
        payment_mint,
        payment_token_account,
        equity_token_account,
        expires_at,
        created_at,
        price_source: market_price.source,
        price_updated_at: market_price.as_of,
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
            + crate::features::settlement::direct_purchases::quotes::ports::QuotePolicy
            + crate::features::identity::access::ports::RateLimit
            + crate::features::issuer::assets::issuance::setup::ports::SetupLookup
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
