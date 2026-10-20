use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::app::support::unix_now;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::features::issuer::assets::drafts::usecases::asset_draft_response;
use crate::features::issuer::organizations::usecases::require_approved_issuer;
use identity::AuthError;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::organizations::models::IssuerOrganization;

pub(crate) async fn submit_asset_for_demo_review(
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
    if asset.draft.metadata.metadata_uri.is_none() {
        let organization = state
            .organizations()
            .get_issuer_organization(&user.user_id)
            .await
            .map_err(ApplicationError::from)?
            .ok_or_else(|| ApplicationError::from(AuthError::OnboardingIncomplete))?;
        let document = asset_metadata_document(&asset, &organization);
        let metadata_uri = state
            .metadata_storage()
            .publish(&asset.asset_id, &document)
            .await
            .map_err(ApplicationError::from)?;
        state
            .assets()
            .set_asset_metadata_uri(&asset.asset_id, &user.user_id, metadata_uri, unix_now())
            .await
            .map_err(ApplicationError::from)?;
    }
    let approved = state
        .assets()
        .approve_asset_for_demo_setup(&asset_id, &user.user_id, unix_now())
        .await
        .map_err(ApplicationError::from)?;
    Ok(asset_draft_response(approved))
}

pub(crate) fn asset_metadata_document(
    asset: &AssetDraftRecord,
    organization: &IssuerOrganization,
) -> serde_json::Value {
    let draft = &asset.draft;
    let mut properties = serde_json::Map::new();
    for property in &draft.metadata.additional_metadata {
        if matches!(
            property.key.trim().to_ascii_lowercase().as_str(),
            "underlying" | "underlying_reference"
        ) {
            continue;
        }
        properties.insert(
            property.key.clone(),
            serde_json::Value::String(property.value.clone()),
        );
    }
    properties.insert("ratio".into(), serde_json::json!(draft.representation));
    properties.insert("asset_id".into(), serde_json::json!(asset.asset_id));
    properties.insert("asset_type".into(), serde_json::json!(draft.asset_type));
    properties.insert("market".into(), serde_json::json!(draft.market));
    properties.insert("share_class".into(), serde_json::json!(draft.share_class));
    properties.insert(
        "representation".into(),
        serde_json::json!(draft.representation),
    );
    properties.insert(
        "rights_description".into(),
        serde_json::json!(draft.rights_description),
    );
    properties.insert("decimals".into(), serde_json::json!(draft.decimals));
    properties.insert(
        "authorized_units".into(),
        serde_json::json!(draft.authorized_units.to_string()),
    );
    properties.insert(
        "settlement_currency".into(),
        serde_json::json!(draft.settlement_currency),
    );
    properties.insert(
        "issuer_name".into(),
        serde_json::json!(organization.legal_name),
    );
    properties.insert("disclosure".into(), serde_json::json!(draft.disclosure));
    properties.insert("schema_version".into(), serde_json::json!("1.0"));
    let mut document = serde_json::Map::new();
    document.insert("name".into(), serde_json::json!(draft.metadata.name));
    document.insert("symbol".into(), serde_json::json!(draft.metadata.symbol));
    document.insert(
        "description".into(),
        serde_json::json!(draft.metadata.description),
    );
    if let Some(image) = &draft.metadata.image_uri {
        document.insert("image".into(), serde_json::json!(image));
    }
    if let Some(external_url) = &draft.metadata.external_url {
        document.insert("external_url".into(), serde_json::json!(external_url));
    }
    document.insert("properties".into(), serde_json::Value::Object(properties));
    serde_json::Value::Object(document)
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::assets::drafts::ports::AssetStore
    + crate::features::issuer::assets::media::ports::MetadataProvider
    + crate::features::issuer::organizations::ports::OrganizationStore
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::assets::drafts::ports::AssetStore
            + crate::features::issuer::assets::media::ports::MetadataProvider
            + crate::features::issuer::organizations::ports::OrganizationStore
            + ?Sized,
    > Ports for T
{
}
