use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

/// `{ "data": ... }` envelope the frontend `apiClient` unwraps.
#[derive(Debug, Serialize)]
pub struct Data<T> {
    pub data: T,
}
