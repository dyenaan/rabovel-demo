use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::app::support::unix_now;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::features::issuer::assets::drafts::usecases::asset_draft_response;
use crate::features::issuer::assets::drafts::usecases::ensure_ticker_tradable;
use crate::features::issuer::assets::issuance::inventory::usecases::inventory_context;
use identity::AuthError;

pub(crate) async fn publish_asset_listing(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
) -> Result<AssetDraftResponse, ApplicationError> {
    let (asset, operation) = inventory_context(state, &user.user_id, &asset_id).await?;
    ensure_ticker_tradable(state, &asset).await?;
    if asset
        .draft
        .backing
        .as_ref()
        .is_none_or(|backing| backing.verification_status != "verified")
    {
        return Err(ApplicationError {
            kind: FailureKind::Conflict,
            error: AuthError::PolicyViolation("verified backing is required before listing".into()),
        });
    }
    let inventory = state
        .inventory()
        .initial_inventory(&asset, &operation, false)
        .await
        .map_err(|_| ApplicationError::from(AuthError::ServiceUnavailable))?;
    if !inventory.issuance_complete {
        return Err(ApplicationError {
            kind: FailureKind::Conflict,
            error: AuthError::PolicyViolation(
                "initial inventory must be issued before listing".into(),
            ),
        });
    }
    let record = state
        .assets()
        .publish_asset_listing(&asset_id, &user.user_id, unix_now())
        .await
        .map_err(ApplicationError::from)?;
    Ok(asset_draft_response(record))
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
