use crate::http::models::HealthResponse;
use axum::response::IntoResponse;
use axum::response::Json;

#[utoipa::path(get, path = "/health", responses((status = 200, body = HealthResponse)))]
pub(crate) async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}
