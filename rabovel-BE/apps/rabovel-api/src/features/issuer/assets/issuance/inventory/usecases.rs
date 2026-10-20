use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::drafts::usecases::owned_setup_asset;
use crate::features::issuer::assets::issuance::inventory::dto::InitialInventoryResponse;
use identity::AuthError;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::issuance::inventory::models::InitialInventory;
use issuer_assets::assets::issuance::setup::models::AssetSetupOperation;

pub(crate) fn initial_inventory_response(inventory: InitialInventory) -> InitialInventoryResponse {
    InitialInventoryResponse {
        network: inventory.network,
        mint_address: inventory.mint_address,
        settlement_wallet: inventory.settlement_wallet,
        token_account: inventory.token_account,
        authorized_units: inventory.authorized_units,
        supply: inventory.supply,
        inventory_balance: inventory.inventory_balance,
        wallet_allowlisted: inventory.wallet_allowlisted,
        token_account_ready: inventory.token_account_ready,
        issuance_complete: inventory.issuance_complete,
        signatures: inventory.signatures,
    }
}

pub(crate) async fn inventory_context(
    state: &impl Ports,
    user_id: &str,
    asset_id: &str,
) -> Result<(AssetDraftRecord, AssetSetupOperation), ApplicationError> {
    let asset = owned_setup_asset(state, user_id, asset_id).await?;
    let operation = state
        .setups()
        .get_asset_setup_operation(asset_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::Conflict,
            error: AuthError::PolicyViolation("asset mint setup has not started".into()),
        })?;
    if operation.status != "confirmed" {
        return Err(ApplicationError {
            kind: FailureKind::Conflict,
            error: AuthError::PolicyViolation("asset mint setup is not confirmed".into()),
        });
    }
    Ok((asset, operation))
}

pub(crate) async fn get_initial_inventory(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
) -> Result<InitialInventoryResponse, ApplicationError> {
    let (asset, operation) = inventory_context(state, &user.user_id, &asset_id).await?;
    state
        .inventory()
        .initial_inventory(&asset, &operation, false)
        .await
        .map(initial_inventory_response)
        .map_err(|_| ApplicationError::from(AuthError::ServiceUnavailable))
}

pub(crate) async fn issue_initial_inventory(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
) -> Result<InitialInventoryResponse, ApplicationError> {
    let (asset, operation) = inventory_context(state, &user.user_id, &asset_id).await?;
    state
        .inventory()
        .initial_inventory(&asset, &operation, true)
        .await
        .map(initial_inventory_response)
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
        })
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::assets::drafts::ports::AssetStore
    + crate::features::issuer::assets::issuance::inventory::ports::Inventory
    + crate::features::issuer::organizations::ports::OrganizationStore
    + crate::features::issuer::assets::issuance::setup::ports::SetupStore
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::assets::drafts::ports::AssetStore
            + crate::features::issuer::assets::issuance::inventory::ports::Inventory
            + crate::features::issuer::organizations::ports::OrganizationStore
            + crate::features::issuer::assets::issuance::setup::ports::SetupStore
            + ?Sized,
    > Ports for T
{
}
