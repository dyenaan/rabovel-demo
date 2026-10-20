use crate::features::compliance::onboarding::dto::OnboardingStatusResponse;
use crate::features::compliance::onboarding::dto::TermsAcceptanceRequest;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(
    get,
    path = "/onboarding/status",
    responses((status = 200, body = OnboardingStatusResponse), (status = 401, body = ApiErrorResponse))
)]
pub(crate) async fn onboarding_status_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<OnboardingStatusResponse>, ApiError> {
    Ok(Json(
        super::usecases::onboarding_status(&state, user).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/onboarding/terms",
    request_body = TermsAcceptanceRequest,
    responses((status = 200, body = OnboardingStatusResponse), (status = 401, body = ApiErrorResponse))
)]
pub(crate) async fn accept_terms_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<TermsAcceptanceRequest>,
) -> Result<Json<OnboardingStatusResponse>, ApiError> {
    Ok(Json(
        super::usecases::accept_terms(&state, user, request).await?,
    ))
}
