use crate::http::models::ApiErrorResponse;
use axum::response::IntoResponse;
use axum::Json;

pub(crate) use crate::app::errors::ApplicationError as ApiError;
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::from(self.kind),
            Json(ApiErrorResponse {
                code: if self.kind.is_server_error() {
                    "service_unavailable".into()
                } else {
                    self.error.code().to_string()
                },
                message: if self.kind.is_server_error() {
                    "The service is temporarily unavailable.".into()
                } else {
                    self.error.to_string()
                },
            }),
        )
            .into_response()
    }
}
impl From<crate::app::errors::FailureKind> for axum::http::StatusCode {
    fn from(kind: crate::app::errors::FailureKind) -> Self {
        match kind {
            crate::app::errors::FailureKind::Unauthenticated => Self::UNAUTHORIZED,
            crate::app::errors::FailureKind::Forbidden => Self::FORBIDDEN,
            crate::app::errors::FailureKind::Conflict => Self::CONFLICT,
            crate::app::errors::FailureKind::Unavailable => Self::SERVICE_UNAVAILABLE,
            crate::app::errors::FailureKind::Expired => Self::GONE,
            crate::app::errors::FailureKind::NotFound => Self::NOT_FOUND,
            crate::app::errors::FailureKind::RateLimited => Self::TOO_MANY_REQUESTS,
            crate::app::errors::FailureKind::InvalidInput => Self::BAD_REQUEST,
        }
    }
}
