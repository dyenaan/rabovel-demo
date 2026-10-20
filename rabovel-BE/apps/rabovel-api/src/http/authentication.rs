use crate::app::state::GatewayState;
use crate::http::errors::ApiError;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use identity::AuthError;

pub(crate) use crate::features::identity::access::actor::AuthenticatedUser;

#[async_trait]
impl FromRequestParts<GatewayState> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayState,
    ) -> Result<Self, Self::Rejection> {
        let token =
            bearer_token(&parts.headers).ok_or_else(|| ApiError::from(AuthError::Unauthorized))?;
        let session = crate::features::identity::sessions::usecases::authenticate(state, token)
            .await
            .map_err(ApiError::from)?;
        Ok(Self {
            user_id: session.user_id,
            mfa_verified_at: session.mfa_verified_at,
        })
    }
}

pub(crate) fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get("authorization")?.to_str().ok()?;
    value
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}
