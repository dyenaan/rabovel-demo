use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::wallets::queries::usecases::investor_solana_wallet;
use crate::features::settlement::direct_purchases::catalog::dto::InvestorCatalogAssetResponse;
use crate::features::settlement::direct_purchases::catalog::dto::InvestorCatalogResponse;
use identity::AuthError;

pub(crate) async fn investor_catalog(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<InvestorCatalogResponse, ApplicationError> {
    investor_catalog_for_user(state, &user.user_id).await
}

pub(crate) async fn investor_catalog_for_user(
    state: &impl Ports,
    user_id: &str,
) -> Result<InvestorCatalogResponse, ApplicationError> {
    let wallet = investor_solana_wallet(state, user_id).await?;
    let cngn = state
        .payment_assets()
        .cngn_wallet_status(wallet.address())
        .await
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })?;
    let candidates = state
        .asset_queries()
        .list_minted_assets()
        .await
        .map_err(ApplicationError::from)?;
    let mut assets = Vec::new();
    for asset in candidates {
        if asset.draft.listing_status != "live" {
            continue;
        }
        let Some(backing) = asset.draft.backing.clone() else {
            continue;
        };
        let Some(operation) = state
            .setup_queries()
            .get_asset_setup_operation(&asset.asset_id)
            .await
            .map_err(ApplicationError::from)?
        else {
            continue;
        };
        if operation.status != "confirmed" {
            continue;
        }
        let inventory = state
            .inventory()
            .initial_inventory(&asset, &operation, false)
            .await
            .map_err(|_| ApplicationError::from(AuthError::ServiceUnavailable))?;
        if !inventory.issuance_complete {
            continue;
        }
        let position = state
            .positions()
            .investor_position(&operation, wallet.address())
            .await
            .map_err(|_| ApplicationError::from(AuthError::ServiceUnavailable))?;
        let price_decimals = cngn.decimals.unwrap_or(6);
        let price = state
            .price_feed()
            .latest(&asset.draft.ticker, price_decimals)
            .map_err(|message| ApplicationError {
                kind: FailureKind::Unavailable,
                error: AuthError::PolicyViolation(message),
            })?;
        assets.push(InvestorCatalogAssetResponse {
            asset_id: asset.asset_id,
            name: asset.draft.name,
            ticker: asset.draft.ticker,
            description: asset.draft.metadata.description,
            asset_type: asset.draft.asset_type,
            settlement_currency: asset.draft.settlement_currency,
            network: inventory.network,
            mint_address: inventory.mint_address,
            authorized_units: inventory.authorized_units,
            issuer_inventory: inventory.inventory_balance,
            investor_token_account: position.token_account,
            investor_balance: position.balance,
            investor_account_ready: position.account_ready,
            image_uri: asset.draft.metadata.image_uri,
            disclosure: asset.draft.disclosure,
            backing,
            price_per_unit: price.price_base_units.to_string(),
            price_decimals,
            price_source: price.source,
            price_updated_at: price.as_of,
        });
    }
    Ok(InvestorCatalogResponse {
        wallet_address: wallet.address().into(),
        cngn,
        assets,
    })
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::assets::listings::ports::PublishedAssets
    + crate::features::issuer::assets::issuance::inventory::ports::Inventory
    + crate::features::payments::cngn::ports::PaymentProvider
    + crate::features::issuer::assets::issuance::inventory::ports::Positions
    + crate::features::market_data::prices::ports::ReferencePrices
    + crate::features::issuer::assets::issuance::setup::ports::SetupLookup
    + crate::features::identity::wallets::queries::ports::WalletLookup
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::assets::listings::ports::PublishedAssets
            + crate::features::issuer::assets::issuance::inventory::ports::Inventory
            + crate::features::payments::cngn::ports::PaymentProvider
            + crate::features::issuer::assets::issuance::inventory::ports::Positions
            + crate::features::market_data::prices::ports::ReferencePrices
            + crate::features::issuer::assets::issuance::setup::ports::SetupLookup
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
