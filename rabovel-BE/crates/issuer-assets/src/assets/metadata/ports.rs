use async_trait::async_trait;
use identity::AuthError;
use serde_json::Value;

#[async_trait]
pub trait MetadataStorage: Send + Sync {
    async fn publish(&self, asset_id: &str, document: &Value) -> Result<String, AuthError>;
    async fn create_signed_image_upload(&self, object_path: &str) -> Result<String, AuthError>;
    async fn confirm_public_image(
        &self,
        object_path: &str,
        content_type: &str,
        max_bytes: u64,
    ) -> Result<String, AuthError>;
}
