use crate::sessions::models::SessionRecord;
use crate::sessions::ports::SessionRepository;
use crate::wallets::linking::models::WalletChallengeRecord;
use crate::wallets::linking::ports::ChallengeRepository;
use crate::AuthError;
use async_trait::async_trait;
use redis::AsyncCommands;
use serde::Deserialize;
use serde::Serialize;

fn ser_error(_: serde_json::Error) -> AuthError {
    AuthError::PolicyViolation("failed to encode ephemeral state".to_string())
}
fn redis_error(_e: redis::RedisError) -> AuthError {
    AuthError::ServiceUnavailable
}
/// Redis-backed ephemeral state. Keys are namespaced by kind so a single
/// Redis instance/database can hold all of them without collisions.
pub struct RedisEphemeralStore {
    manager: redis::aio::ConnectionManager,
}
impl RedisEphemeralStore {
    pub async fn connect(redis_url: &str) -> Result<Self, String> {
        let client =
            redis::Client::open(redis_url).map_err(|e| format!("invalid REDIS_URL: {e}"))?;
        let manager = client
            .get_connection_manager()
            .await
            .map_err(|e| format!("failed to connect to Redis: {e}"))?;
        Ok(Self { manager })
    }

    async fn set_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: u64,
    ) -> Result<(), AuthError> {
        let payload = serde_json::to_string(value).map_err(ser_error)?;
        let mut conn = self.manager.clone();
        let _: () = conn
            .set_ex(key, payload, ttl_secs)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, AuthError> {
        let mut conn = self.manager.clone();
        let raw: Option<String> = conn.get(key).await.map_err(redis_error)?;
        raw.map(|s| serde_json::from_str(&s).map_err(ser_error))
            .transpose()
    }

    /// Atomic get-and-delete via Redis 6.2+'s `GETDEL`.
    async fn take_json<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, AuthError> {
        let mut conn = self.manager.clone();
        let raw: Option<String> = redis::cmd("GETDEL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(redis_error)?;
        raw.map(|s| serde_json::from_str(&s).map_err(ser_error))
            .transpose()
    }

    async fn exists(&self, key: &str) -> Result<bool, AuthError> {
        let mut conn = self.manager.clone();
        let count: u32 = conn.exists(key).await.map_err(redis_error)?;
        Ok(count > 0)
    }

    async fn mark_used(&self, key: &str, ttl_secs: u64) -> Result<(), AuthError> {
        let mut conn = self.manager.clone();
        let _: () = conn.set_ex(key, "1", ttl_secs).await.map_err(redis_error)?;
        Ok(())
    }
}

#[async_trait]
impl SessionRepository for RedisEphemeralStore {
    async fn create_session(
        &self,
        token_hash: &str,
        record: SessionRecord,
        ttl_secs: u64,
    ) -> Result<(), AuthError> {
        self.set_json(&format!("session:{token_hash}"), &record, ttl_secs)
            .await
    }
    async fn get_session(&self, token_hash: &str) -> Result<Option<SessionRecord>, AuthError> {
        self.get_json(&format!("session:{token_hash}")).await
    }
    async fn delete_session(&self, token_hash: &str) -> Result<(), AuthError> {
        let mut conn = self.manager.clone();
        let _: u32 = redis::AsyncCommands::del(&mut conn, format!("session:{token_hash}"))
            .await
            .map_err(redis_error)?;
        Ok(())
    }
}

#[async_trait]
impl ChallengeRepository for RedisEphemeralStore {
    async fn put_wallet_challenge(
        &self,
        challenge_id: &str,
        challenge: WalletChallengeRecord,
        ttl_secs: u64,
    ) -> Result<(), AuthError> {
        self.set_json(
            &format!("wallet_challenge:{challenge_id}"),
            &challenge,
            ttl_secs,
        )
        .await
    }
    async fn take_wallet_challenge(
        &self,
        challenge_id: &str,
    ) -> Result<Option<WalletChallengeRecord>, AuthError> {
        self.take_json(&format!("wallet_challenge:{challenge_id}"))
            .await
    }
    async fn is_wallet_challenge_used(&self, challenge_id: &str) -> Result<bool, AuthError> {
        self.exists(&format!("wallet_challenge_used:{challenge_id}"))
            .await
    }
    async fn mark_wallet_challenge_used(
        &self,
        challenge_id: &str,
        ttl_secs: u64,
    ) -> Result<(), AuthError> {
        self.mark_used(&format!("wallet_challenge_used:{challenge_id}"), ttl_secs)
            .await
    }
}
