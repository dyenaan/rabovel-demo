use super::models::SessionRecord;
use crate::AuthError;
use async_trait::async_trait;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create_session(
        &self,
        token_hash: &str,
        record: SessionRecord,
        ttl_secs: u64,
    ) -> Result<(), AuthError>;
    async fn get_session(&self, token_hash: &str) -> Result<Option<SessionRecord>, AuthError>;

    async fn delete_session(&self, token_hash: &str) -> Result<(), AuthError>;
}
