#![cfg(debug_assertions)]
use crate::features::compliance::kyc::cases::dto::SandboxAdvanceRequest;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;

#[cfg(debug_assertions)]
pub(crate) async fn kyc_sandbox_advance_handler(
    State(state): State<super::context::FeatureContext>,
    Json(request): Json<SandboxAdvanceRequest>,
) -> Result<StatusCode, ApiError> {
    super::usecases::kyc_sandbox_advance(&state, request).await?;
    Ok(StatusCode::OK)
}
