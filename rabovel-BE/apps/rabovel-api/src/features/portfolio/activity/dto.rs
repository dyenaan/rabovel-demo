use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEventResponse {
    pub event_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub title: String,
    pub description: String,
    pub timestamp: String,
}
