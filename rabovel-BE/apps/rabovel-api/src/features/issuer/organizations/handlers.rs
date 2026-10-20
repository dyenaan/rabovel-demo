use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::organizations::dto::IssuerOnboardingRequest;
use crate::features::issuer::organizations::dto::IssuerOverviewResponse;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(get, path = "/issuer/overview", responses((status = 200, body = IssuerOverviewResponse), (status = 401, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse)))]
pub(crate) async fn issuer_overview_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<IssuerOverviewResponse>, ApiError> {
    Ok(Json(super::usecases::issuer_overview(&state, user).await?))
}

#[utoipa::path(post, path = "/issuer/onboarding", request_body = IssuerOnboardingRequest, responses((status = 200, body = IssuerOverviewResponse), (status = 400, body = ApiErrorResponse), (status = 401, body = ApiErrorResponse), (status = 403, body = ApiErrorResponse)))]
pub(crate) async fn issuer_onboarding_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Json(request): Json<IssuerOnboardingRequest>,
) -> Result<Json<IssuerOverviewResponse>, ApiError> {
    Ok(Json(
        super::usecases::issuer_onboarding(&state, user, request).await?,
    ))
}
