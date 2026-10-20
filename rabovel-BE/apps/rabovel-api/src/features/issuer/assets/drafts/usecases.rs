use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::app::support::random_hex;
use crate::app::support::unix_now;
use crate::features::catalog::instruments as instrument_provider;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::features::issuer::assets::drafts::dto::CreateAssetDraftRequest;
use crate::features::issuer::organizations::usecases::require_approved_issuer;
use identity::AuthError;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::drafts::models::NewAssetDraft;

pub(crate) fn asset_draft_response(record: AssetDraftRecord) -> AssetDraftResponse {
    AssetDraftResponse {
        asset_id: record.asset_id,
        issuer_user_id: record.issuer_user_id,
        status: record.status,
        issued_units: "0".into(),
        mint_address: record.mint_address,
        created_at: record.created_at,
        updated_at: record.updated_at,
        draft: record.draft,
    }
}

pub(crate) async fn create_asset_draft(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: CreateAssetDraftRequest,
) -> Result<AssetDraftResponse, ApplicationError> {
    require_approved_issuer(state, &user.user_id).await?;
    let draft = asset_draft_from_request(request)?;
    reject_ngx_listed_ticker(&draft.ticker)?;
    let now = unix_now();
    let record = AssetDraftRecord {
        asset_id: format!("ast_{}", random_hex(12).map_err(ApplicationError::from)?),
        issuer_user_id: user.user_id,
        canonical_key: draft.canonical_key(),
        draft,
        status: "draft".into(),
        created_at: now,
        updated_at: now,
        mint_address: None,
    };
    state
        .assets()
        .create_asset_draft(record.clone())
        .await
        .map_err(ApplicationError::from)?;
    Ok(asset_draft_response(record))
}

/// Issuer tickers share one namespace with the NGX-listed catalog.
pub(crate) fn reject_ngx_listed_ticker(ticker: &str) -> Result<(), ApplicationError> {
    if instrument_provider::is_ngx_listed(ticker) {
        return Err(ApplicationError {
            kind: FailureKind::Conflict,
            error: AuthError::PolicyViolation(format!(
                "ticker {ticker} is already an NGX-listed instrument; choose a distinct ticker"
            )),
        });
    }
    Ok(())
}

/// A live listing joins the tradable catalog, so its ticker must be valid and
/// unused by any other tradable instrument.
pub(crate) async fn ensure_ticker_tradable(
    state: &impl Ports,
    asset: &AssetDraftRecord,
) -> Result<(), ApplicationError> {
    let ticker = &asset.draft.ticker;
    if !issuer_assets::assets::drafts::models::is_valid_ticker(ticker) {
        return Err(ApplicationError {
            kind: FailureKind::Conflict,
            error: AuthError::PolicyViolation(format!(
                "ticker {ticker} does not follow the NGX ticker format; create a new asset with a ticker like DANGCEM"
            )),
        });
    }
    reject_ngx_listed_ticker(ticker)?;
    let taken = state
        .assets()
        .list_minted_assets()
        .await
        .map_err(ApplicationError::from)?
        .into_iter()
        .any(|other| {
            other.asset_id != asset.asset_id
                && other.draft.listing_status == "live"
                && other.draft.ticker.eq_ignore_ascii_case(ticker)
        });
    if taken {
        return Err(ApplicationError {
            kind: FailureKind::Conflict,
            error: AuthError::PolicyViolation(format!(
                "ticker {ticker} is already listed by another issuer"
            )),
        });
    }
    Ok(())
}

pub(crate) fn asset_draft_from_request(
    request: CreateAssetDraftRequest,
) -> Result<NewAssetDraft, ApplicationError> {
    if request.metadata.metadata_uri.is_some() {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "metadata_uri is assigned by Rabovel after publishing".into(),
        )));
    }
    let authorized_units = request.authorized_units.parse::<u64>().map_err(|_| {
        ApplicationError::from(AuthError::PolicyViolation(
            "authorized_units must be a positive whole-number string".into(),
        ))
    })?;
    NewAssetDraft {
        instrument_code: request.instrument_code,
        name: request.name,
        ticker: request.ticker,
        market: request.market,
        share_class: request.share_class,
        asset_type: request.asset_type,
        decimals: request.decimals,
        authorized_units,
        settlement_currency: request.settlement_currency,
        representation: request.representation,
        rights_description: request.rights_description,
        disclosure: request.disclosure,
        metadata: request.metadata,
        backing: None,
        listing_status: "not_listed".into(),
    }
    .validate_and_normalize()
    .map_err(|message| ApplicationError::from(AuthError::PolicyViolation(message)))
}

pub(crate) async fn list_asset_drafts(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<Vec<AssetDraftResponse>, ApplicationError> {
    require_approved_issuer(state, &user.user_id).await?;
    let assets = state
        .assets()
        .list_asset_drafts(&user.user_id)
        .await
        .map_err(ApplicationError::from)?;
    Ok(assets.into_iter().map(asset_draft_response).collect())
}

pub(crate) async fn get_asset_draft(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
) -> Result<AssetDraftResponse, ApplicationError> {
    require_approved_issuer(state, &user.user_id).await?;
    let asset = state
        .assets()
        .get_asset_draft(&asset_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::NotFound,
            error: AuthError::PolicyViolation("asset draft not found".into()),
        })?;
    if asset.issuer_user_id != user.user_id {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }
    Ok(asset_draft_response(asset))
}

pub(crate) async fn update_asset_draft(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
    request: CreateAssetDraftRequest,
) -> Result<AssetDraftResponse, ApplicationError> {
    require_approved_issuer(state, &user.user_id).await?;
    let existing = state
        .assets()
        .get_asset_draft(&asset_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::NotFound,
            error: AuthError::PolicyViolation("asset draft not found".into()),
        })?;
    if existing.issuer_user_id != user.user_id || existing.status != "draft" {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }
    let draft = asset_draft_from_request(request)?;
    reject_ngx_listed_ticker(&draft.ticker)?;
    let updated = AssetDraftRecord {
        canonical_key: draft.canonical_key(),
        draft,
        updated_at: unix_now(),
        ..existing
    };
    state
        .assets()
        .update_asset_draft(updated.clone())
        .await
        .map_err(ApplicationError::from)?;
    Ok(asset_draft_response(updated))
}

pub(crate) async fn delete_asset_draft(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
) -> Result<(), ApplicationError> {
    require_approved_issuer(state, &user.user_id).await?;
    state
        .assets()
        .delete_asset_draft(&asset_id, &user.user_id)
        .await
        .map_err(ApplicationError::from)?;
    Ok(())
}

pub(crate) async fn owned_setup_asset(
    state: &impl Ports,
    user_id: &str,
    asset_id: &str,
) -> Result<AssetDraftRecord, ApplicationError> {
    require_approved_issuer(state, user_id).await?;
    let asset = state
        .assets()
        .get_asset_draft(asset_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::NotFound,
            error: AuthError::PolicyViolation("asset draft not found".into()),
        })?;
    if asset.issuer_user_id != user_id {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }
    Ok(asset)
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::assets::drafts::ports::AssetStore
    + crate::features::issuer::organizations::ports::OrganizationStore
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::assets::drafts::ports::AssetStore
            + crate::features::issuer::organizations::ports::OrganizationStore
            + ?Sized,
    > Ports for T
{
}
