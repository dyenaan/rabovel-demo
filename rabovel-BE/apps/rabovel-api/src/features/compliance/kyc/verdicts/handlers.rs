use crate::http::errors::ApiError;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;

pub(crate) async fn kyc_webhook_handler(
    State(state): State<super::context::FeatureContext>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<StatusCode, ApiError> {
    let signature = headers
        .get("x-kyc-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::from(kyc::KycError::InvalidSignature))?;
    super::usecases::kyc_webhook(&state, signature, &body).await?;
    Ok(StatusCode::OK)
}
