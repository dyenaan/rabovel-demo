use crate::features::compliance::kyc::cases::dto::KycCaseRequest;
use crate::features::compliance::kyc::cases::dto::KycCaseResponse;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

pub(crate) async fn submit_kyc_case_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<KycCaseRequest>,
) -> Result<Json<KycCaseResponse>, ApiError> {
    Ok(Json(
        super::usecases::submit_kyc_case(&state, user, request).await?,
    ))
}
