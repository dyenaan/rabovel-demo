use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::app::support::random_hex;
use crate::app::support::unix_now;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::drafts::dto::AssetDraftResponse;
use crate::features::issuer::assets::drafts::usecases::asset_draft_response;
use crate::features::issuer::assets::drafts::usecases::owned_setup_asset;
use crate::features::issuer::assets::media::dto::ConfirmImageUploadRequest;
use crate::features::issuer::assets::media::dto::ImageUploadRequest;
use crate::features::issuer::assets::media::dto::ImageUploadTicket;
use crate::features::issuer::organizations::usecases::require_approved_issuer;
use identity::AuthError;

pub(crate) const MAX_ASSET_IMAGE_BYTES: u64 = 5 * 1024 * 1024;

pub(crate) async fn create_asset_image_upload(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
    request: ImageUploadRequest,
) -> Result<ImageUploadTicket, ApplicationError> {
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
    if asset.issuer_user_id != user.user_id || asset.status != "draft" {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }
    if request.size == 0 || request.size > MAX_ASSET_IMAGE_BYTES {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "image must be between 1 byte and 5 MB".into(),
        )));
    }
    let extension = match request.content_type.as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => {
            return Err(ApplicationError::from(AuthError::PolicyViolation(
                "image must be PNG, JPEG, or WebP".into(),
            )))
        }
    };
    let object_path = format!(
        "assets/{asset_id}/images/{}.{}",
        random_hex(16).map_err(ApplicationError::from)?,
        extension
    );
    let signed_url = state
        .metadata_storage()
        .create_signed_image_upload(&object_path)
        .await
        .map_err(ApplicationError::from)?;
    Ok(ImageUploadTicket {
        signed_url,
        object_path,
    })
}

pub(crate) async fn confirm_asset_image_upload(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
    request: ConfirmImageUploadRequest,
) -> Result<AssetDraftResponse, ApplicationError> {
    let asset = owned_setup_asset(state, &user.user_id, &asset_id).await?;
    if asset.status != "draft" {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }
    let expected_prefix = format!("assets/{asset_id}/images/");
    if !request.object_path.starts_with(&expected_prefix) {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }
    if !matches!(
        request.content_type.as_str(),
        "image/png" | "image/jpeg" | "image/webp"
    ) {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "image must be PNG, JPEG, or WebP".into(),
        )));
    }
    let image_uri = state
        .metadata_storage()
        .confirm_public_image(
            &request.object_path,
            &request.content_type,
            MAX_ASSET_IMAGE_BYTES,
        )
        .await
        .map_err(ApplicationError::from)?;
    let asset = state
        .assets()
        .set_asset_image_uri(&asset_id, &user.user_id, image_uri, unix_now())
        .await
        .map_err(ApplicationError::from)?;
    Ok(asset_draft_response(asset))
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
