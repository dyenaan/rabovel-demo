use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::app::support::random_hex;
use crate::app::support::unix_now;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::assets::drafts::usecases::owned_setup_asset;
use crate::features::issuer::assets::issuance::setup::dto::AssetSetupOperationResponse;
use crate::features::issuer::assets::issuance::setup::dto::SetupStageResponse;
use identity::AuthError;
use issuer_assets::assets::issuance::setup::models::AssetSetupOperation;
use issuer_assets::assets::issuance::setup::models::SetupStageRecord;

pub(crate) fn asset_setup_response(operation: AssetSetupOperation) -> AssetSetupOperationResponse {
    AssetSetupOperationResponse {
        operation_id: operation.operation_id,
        asset_id: operation.asset_id,
        network: operation.network,
        mint_address: operation.mint_address,
        status: operation.status,
        stages: operation
            .stages
            .into_iter()
            .map(|stage| SetupStageResponse {
                stage: stage.stage,
                status: stage.status,
                signatures: stage.signatures,
                verified_at: stage.verified_at,
            })
            .collect(),
        mint_config_address: operation.mint_config_address,
        allow_list_address: operation.allow_list_address,
        block_list_address: operation.block_list_address,
        thaw_extra_metas_address: operation.thaw_extra_metas_address,
        error: operation.error,
        created_at: operation.created_at,
        updated_at: operation.updated_at,
    }
}

pub(crate) async fn get_asset_setup_operation(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
) -> Result<AssetSetupOperationResponse, ApplicationError> {
    owned_setup_asset(state, &user.user_id, &asset_id).await?;
    let operation = state
        .setups()
        .get_asset_setup_operation(&asset_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::NotFound,
            error: AuthError::PolicyViolation("asset setup operation not found".into()),
        })?;
    Ok(asset_setup_response(operation))
}

pub(crate) async fn start_asset_setup(
    state: &impl Ports,
    user: AuthenticatedUser,
    asset_id: String,
) -> Result<AssetSetupOperationResponse, ApplicationError> {
    let asset = owned_setup_asset(state, &user.user_id, &asset_id).await?;
    if !matches!(asset.status.as_str(), "approved_for_setup" | "minted") {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "asset must be approved before mint setup".into(),
        )));
    }
    if asset.draft.metadata.metadata_uri.is_none() {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "hosted metadata is required before mint setup".into(),
        )));
    }
    if !state.mint_setup().is_configured() {
        return Err(ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation("Solana mint setup is not configured".into()),
        });
    }
    let existing = state
        .setups()
        .get_asset_setup_operation(&asset_id)
        .await
        .map_err(ApplicationError::from)?;
    let mut operation = if let Some(mut existing) = existing {
        if existing.status != "failed" {
            return Ok(asset_setup_response(existing));
        }
        existing.status = "running".into();
        existing.error = None;
        existing.updated_at = unix_now();
        state
            .setups()
            .save_asset_setup_operation(existing.clone())
            .await
            .map_err(ApplicationError::from)?;
        existing
    } else {
        let now = unix_now();
        let operation_id = format!("setup_{}", random_hex(12).map_err(ApplicationError::from)?);
        let proposed = AssetSetupOperation {
            operation_id: operation_id.clone(),
            asset_id: asset.asset_id.clone(),
            issuer_user_id: user.user_id.clone(),
            network: state.mint_setup().network().to_string(),
            mint_address: state.mint_setup().mint_address(&operation_id),
            status: "running".into(),
            stages: vec![SetupStageRecord {
                stage: "metadata_ready".into(),
                status: "verified".into(),
                signatures: vec![],
                verified_at: Some(now),
            }],
            mint_config_address: None,
            allow_list_address: None,
            block_list_address: None,
            thaw_extra_metas_address: None,
            error: None,
            created_at: now,
            updated_at: now,
        };
        let created = state
            .setups()
            .create_asset_setup_operation(proposed)
            .await
            .map_err(ApplicationError::from)?;
        if created.operation_id != operation_id {
            return Ok(asset_setup_response(created));
        }
        created
    };

    match state.mint_setup().execute(&asset, &operation).await {
        Ok(execution) => {
            let verified_at = unix_now();
            operation.status = "confirmed".into();
            operation.updated_at = verified_at;
            operation.mint_config_address = Some(execution.mint_config_address);
            operation.allow_list_address = Some(execution.allow_list_address);
            operation.block_list_address = Some(execution.block_list_address);
            operation.thaw_extra_metas_address = Some(execution.thaw_extra_metas_address);
            operation.stages.extend([
                SetupStageRecord {
                    stage: "mint_verified".into(),
                    status: "verified".into(),
                    signatures: execution.mint_signature.into_iter().collect(),
                    verified_at: Some(verified_at),
                },
                SetupStageRecord {
                    stage: "shared_lists_verified".into(),
                    status: "verified".into(),
                    signatures: execution.shared_lists_signature.into_iter().collect(),
                    verified_at: Some(verified_at),
                },
                SetupStageRecord {
                    stage: "acl_verified".into(),
                    status: "verified".into(),
                    signatures: execution.acl_signatures,
                    verified_at: Some(verified_at),
                },
            ]);
            state
                .setups()
                .save_asset_setup_operation(operation.clone())
                .await
                .map_err(ApplicationError::from)?;
            state
                .assets()
                .mark_asset_minted(
                    &asset_id,
                    &user.user_id,
                    &operation.mint_address,
                    verified_at,
                )
                .await
                .map_err(ApplicationError::from)?;
        }
        Err(error) => {
            operation.status = if error.reconciliation_required {
                "reconciliation_required"
            } else {
                "failed"
            }
            .into();
            operation.error = Some(error.message);
            operation.updated_at = unix_now();
            state
                .setups()
                .save_asset_setup_operation(operation.clone())
                .await
                .map_err(ApplicationError::from)?;
        }
    }
    Ok(asset_setup_response(operation))
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::assets::drafts::ports::AssetStore
    + crate::features::issuer::assets::issuance::setup::ports::IssuanceProvider
    + crate::features::issuer::organizations::ports::OrganizationStore
    + crate::features::issuer::assets::issuance::setup::ports::SetupStore
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::assets::drafts::ports::AssetStore
            + crate::features::issuer::assets::issuance::setup::ports::IssuanceProvider
            + crate::features::issuer::organizations::ports::OrganizationStore
            + crate::features::issuer::assets::issuance::setup::ports::SetupStore
            + ?Sized,
    > Ports for T
{
}
