use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::sessions::dto::CurrentUserResponse;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::Json;

#[utoipa::path(get, path = "/auth/me", responses((status = 200, body = CurrentUserResponse), (status = 401, body = ApiErrorResponse)))]
pub(crate) async fn auth_me_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<CurrentUserResponse>, ApiError> {
    Ok(Json(super::usecases::auth_me(&state, user).await?))
}

#[utoipa::path(post, path = "/auth/logout", responses((status = 204), (status = 401, body = ApiErrorResponse)))]
pub(crate) async fn logout_handler(
    State(state): State<super::context::FeatureContext>,
    headers: HeaderMap,
    _user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let token = crate::http::authentication::bearer_token(&headers)
        .ok_or_else(|| ApiError::from(identity::AuthError::Unauthorized))?;
    super::usecases::logout(&state, token, _user).await?;
    Ok(StatusCode::NO_CONTENT)
}
