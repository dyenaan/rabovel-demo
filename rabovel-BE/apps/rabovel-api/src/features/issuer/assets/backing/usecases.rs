use crate::app::errors::ApplicationError;
use crate::app::support::unix_now;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::backing::dto::SubmitBackingRequest;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::features::issuer::assets::drafts::usecases::asset_draft_response;
use crate::features::issuer::organizations::usecases::require_approved_issuer;
use identity::AuthError;
use issuer_assets::assets::drafts::models::BackingEvidence;

pub(crate) async fn submit_asset_backing(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
    request: SubmitBackingRequest,
) -> Result<AssetDraftResponse, ApplicationError> {
    require_approved_issuer(state, &user.user_id).await?;
    let summary = request.summary.trim();
    let document_name = request.document_name.trim();
    if summary.len() < 10 || summary.len() > 500 {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "backing summary must be between 10 and 500 characters".into(),
        )));
    }
    if document_name.is_empty() || document_name.len() > 160 {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "backing document name is required and must be at most 160 characters".into(),
        )));
    }
    if !matches!(
        request.content_type.as_str(),
        "application/pdf" | "image/png" | "image/jpeg"
    ) {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "backing document must be a PDF, PNG, or JPEG".into(),
        )));
    }
    if request.size_bytes == 0 || request.size_bytes > 10 * 1024 * 1024 {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "backing document must be between 1 byte and 10 MB".into(),
        )));
    }
    let now = unix_now();
    let record = state
        .assets()
        .save_asset_backing(
            &asset_id,
            &user.user_id,
            BackingEvidence {
                summary: summary.into(),
                document_name: document_name.into(),
                content_type: request.content_type,
                size_bytes: request.size_bytes,
                verification_status: "verified".into(),
                verified_at: now,
            },
            now,
        )
        .await
        .map_err(ApplicationError::from)?;
    Ok(asset_draft_response(record))
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
