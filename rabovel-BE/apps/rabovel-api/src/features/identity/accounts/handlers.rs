use crate::features::identity::accounts::dto::AuthSessionResponse;
use crate::features::identity::accounts::dto::PasswordLoginRequest;
use crate::features::identity::accounts::dto::PasswordRegisterRequest;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(post, path = "/auth/register", request_body = PasswordRegisterRequest, responses((status = 200, body = AuthSessionResponse), (status = 400, body = ApiErrorResponse)))]
pub(crate) async fn password_register_handler(
    State(state): State<super::context::FeatureContext>,
    Json(request): Json<PasswordRegisterRequest>,
) -> Result<Json<AuthSessionResponse>, ApiError> {
    Ok(Json(
        super::usecases::password_register(&state, request).await?,
    ))
}

#[utoipa::path(post, path = "/auth/login", request_body = PasswordLoginRequest, responses((status = 200, body = AuthSessionResponse), (status = 401, body = ApiErrorResponse)))]
pub(crate) async fn password_login_handler(
    State(state): State<super::context::FeatureContext>,
    Json(request): Json<PasswordLoginRequest>,
) -> Result<Json<AuthSessionResponse>, ApiError> {
    Ok(Json(
        super::usecases::password_login(&state, request).await?,
    ))
}
