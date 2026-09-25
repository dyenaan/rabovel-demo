use async_trait::async_trait;
use domain::auth::AuthError;
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

pub struct DisabledMetadataStorage;

#[async_trait]
impl MetadataStorage for DisabledMetadataStorage {
    async fn publish(&self, _asset_id: &str, _document: &Value) -> Result<String, AuthError> {
        Err(AuthError::PolicyViolation(
            "metadata storage is not configured".into(),
        ))
    }
    async fn create_signed_image_upload(&self, _: &str) -> Result<String, AuthError> {
        Err(AuthError::PolicyViolation(
            "metadata storage is not configured".into(),
        ))
    }
    async fn confirm_public_image(&self, _: &str, _: &str, _: u64) -> Result<String, AuthError> {
        Err(AuthError::PolicyViolation(
            "metadata storage is not configured".into(),
        ))
    }
}

pub struct SupabaseMetadataStorage {
    project_url: String,
    secret_key: String,
    client: reqwest::Client,
}

impl SupabaseMetadataStorage {
    pub fn new(project_url: String, secret_key: String) -> Result<Self, String> {
        let project_url = project_url.trim_end_matches('/').to_string();
        let parsed = reqwest::Url::parse(&project_url)
            .map_err(|_| "SUPABASE_URL must be an absolute URL".to_string())?;
        if parsed.scheme() != "https" && parsed.host_str() != Some("localhost") {
            return Err("SUPABASE_URL must use HTTPS outside localhost".into());
        }
        if secret_key.trim().is_empty() {
            return Err("SUPABASE_SECRET_KEY cannot be empty".into());
        }
        Ok(Self {
            project_url,
            secret_key,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl MetadataStorage for SupabaseMetadataStorage {
    async fn publish(&self, asset_id: &str, document: &Value) -> Result<String, AuthError> {
        if !asset_id.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        }) {
            return Err(AuthError::PolicyViolation(
                "invalid asset id for metadata storage".into(),
            ));
        }
        let object_path = format!("assets/{asset_id}/metadata.json");
        let upload_url = format!(
            "{}/storage/v1/object/metadata/{object_path}",
            self.project_url
        );
        let response = self
            .client
            .put(upload_url)
            .header("apikey", &self.secret_key)
            .header("x-upsert", "true")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(document)
            .send()
            .await
            .map_err(|_| AuthError::PolicyViolation("metadata upload failed".into()))?;
        if !response.status().is_success() {
            return Err(AuthError::PolicyViolation(format!(
                "metadata upload failed with status {}",
                response.status()
            )));
        }
        Ok(format!(
            "{}/storage/v1/object/public/metadata/{object_path}",
            self.project_url
        ))
    }
    async fn create_signed_image_upload(&self, object_path: &str) -> Result<String, AuthError> {
        let response = self
            .client
            .post(format!(
                "{}/storage/v1/object/upload/sign/metadata/{object_path}",
                self.project_url
            ))
            .header("apikey", &self.secret_key)
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|_| AuthError::PolicyViolation("failed to create image upload URL".into()))?;
        if !response.status().is_success() {
            return Err(AuthError::PolicyViolation(format!(
                "failed to create image upload URL with status {}",
                response.status()
            )));
        }
        let body: Value = response
            .json()
            .await
            .map_err(|_| AuthError::PolicyViolation("invalid signed upload response".into()))?;
        let relative = body.get("url").and_then(Value::as_str).ok_or_else(|| {
            AuthError::PolicyViolation("signed upload response contained no URL".into())
        })?;
        Ok(format!("{}/storage/v1{}", self.project_url, relative))
    }
    async fn confirm_public_image(
        &self,
        object_path: &str,
        content_type: &str,
        max_bytes: u64,
    ) -> Result<String, AuthError> {
        let public_url = format!(
            "{}/storage/v1/object/public/metadata/{object_path}",
            self.project_url
        );
        let response =
            self.client.head(&public_url).send().await.map_err(|_| {
                AuthError::PolicyViolation("failed to verify uploaded image".into())
            })?;
        if !response.status().is_success() {
            return Err(AuthError::PolicyViolation(
                "uploaded image was not found".into(),
            ));
        }
        if response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| !v.starts_with(content_type))
        {
            return Err(AuthError::PolicyViolation(
                "uploaded image content type does not match".into(),
            ));
        }
        if response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .is_some_and(|v| v > max_bytes)
        {
            return Err(AuthError::PolicyViolation(
                "uploaded image is too large".into(),
            ));
        }
        Ok(public_url)
    }
}

pub struct InMemoryMetadataStorage;

#[async_trait]
impl MetadataStorage for InMemoryMetadataStorage {
    async fn publish(&self, asset_id: &str, _document: &Value) -> Result<String, AuthError> {
        Ok(format!(
            "https://metadata.example/assets/{asset_id}/metadata.json"
        ))
    }
    async fn create_signed_image_upload(&self, object_path: &str) -> Result<String, AuthError> {
        Ok(format!("https://upload.example/{object_path}?token=test"))
    }
    async fn confirm_public_image(
        &self,
        object_path: &str,
        _: &str,
        _: u64,
    ) -> Result<String, AuthError> {
        Ok(format!("https://metadata.example/{object_path}"))
    }
}
