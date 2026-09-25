//! Ephemeral, TTL-bound, single-use state: sessions, OIDC pending/used
//! state, and wallet challenges. This is the state that moves to Redis --
//! its native `EXPIRE`/`GETDEL` replace the manual `.retain()` cleanup the
//! original in-memory `HashMap`s needed. Durable state (users, wallets, KYC
//! cases) lives in `crate::repository` instead.

use async_trait::async_trait;
use domain::auth::{AuthError, SupportedChain, WalletProvider};
use serde::{Deserialize, Serialize};

fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("ephemeral state unavailable".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub user_id: String,
    pub mfa_verified_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcPendingRecord {
    pub nonce: String,
    pub pkce_verifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletChallengeRecord {
    pub user_id: String,
    pub expected_address: String,
    pub chain: SupportedChain,
    pub provider: WalletProvider,
    pub message: String,
}

#[async_trait]
pub trait EphemeralStore: Send + Sync {
    async fn create_session(
        &self,
        token_hash: &str,
        record: SessionRecord,
        ttl_secs: u64,
    ) -> Result<(), AuthError>;
    async fn get_session(&self, token_hash: &str) -> Result<Option<SessionRecord>, AuthError>;
    async fn delete_session(&self, token_hash: &str) -> Result<(), AuthError>;

    async fn put_oidc_pending(
        &self,
        state: &str,
        pending: OidcPendingRecord,
        ttl_secs: u64,
    ) -> Result<(), AuthError>;
    /// Atomically removes and returns the pending record for `state`, so a
    /// concurrent replay can never observe (and reuse) it after this call
    /// returns `Some`.
    async fn take_oidc_pending(&self, state: &str) -> Result<Option<OidcPendingRecord>, AuthError>;
    async fn is_oidc_state_used(&self, state: &str) -> Result<bool, AuthError>;
    async fn mark_oidc_state_used(&self, state: &str, ttl_secs: u64) -> Result<(), AuthError>;

    async fn put_wallet_challenge(
        &self,
        challenge_id: &str,
        challenge: WalletChallengeRecord,
        ttl_secs: u64,
    ) -> Result<(), AuthError>;
    async fn take_wallet_challenge(
        &self,
        challenge_id: &str,
    ) -> Result<Option<WalletChallengeRecord>, AuthError>;
    async fn is_wallet_challenge_used(&self, challenge_id: &str) -> Result<bool, AuthError>;
    async fn mark_wallet_challenge_used(
        &self,
        challenge_id: &str,
        ttl_secs: u64,
    ) -> Result<(), AuthError>;
}

pub mod in_memory {
    use std::collections::HashMap;
    use std::sync::RwLock;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn unix_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    struct Entry<T> {
        value: T,
        expires_at: u64,
    }

    /// A straight port of the gateway's original in-memory ephemeral-state
    /// logic (lazy expiry check on read, rather than a background sweep --
    /// functionally equivalent to Redis's TTL from the caller's point of
    /// view: an expired entry simply reads back as absent).
    #[derive(Default)]
    pub struct InMemoryEphemeralStore {
        sessions: RwLock<HashMap<String, Entry<SessionRecord>>>,
        oidc_pending: RwLock<HashMap<String, Entry<OidcPendingRecord>>>,
        used_oidc_states: RwLock<HashMap<String, u64>>,
        wallet_challenges: RwLock<HashMap<String, Entry<WalletChallengeRecord>>>,
        used_wallet_challenges: RwLock<HashMap<String, u64>>,
    }

    #[async_trait]
    impl EphemeralStore for InMemoryEphemeralStore {
        async fn create_session(
            &self,
            token_hash: &str,
            record: SessionRecord,
            ttl_secs: u64,
        ) -> Result<(), AuthError> {
            self.sessions
                .write()
                .map_err(|_| state_unavailable())?
                .insert(
                    token_hash.to_string(),
                    Entry {
                        value: record,
                        expires_at: unix_now().saturating_add(ttl_secs),
                    },
                );
            Ok(())
        }

        async fn get_session(&self, token_hash: &str) -> Result<Option<SessionRecord>, AuthError> {
            let mut sessions = self.sessions.write().map_err(|_| state_unavailable())?;
            let now = unix_now();
            match sessions.get(token_hash) {
                Some(entry) if entry.expires_at > now => Ok(Some(entry.value.clone())),
                Some(_) => {
                    sessions.remove(token_hash);
                    Ok(None)
                }
                None => Ok(None),
            }
        }

        async fn delete_session(&self, token_hash: &str) -> Result<(), AuthError> {
            self.sessions
                .write()
                .map_err(|_| state_unavailable())?
                .remove(token_hash);
            Ok(())
        }

        async fn put_oidc_pending(
            &self,
            state: &str,
            pending: OidcPendingRecord,
            ttl_secs: u64,
        ) -> Result<(), AuthError> {
            let now = unix_now();
            let mut oidc_pending = self.oidc_pending.write().map_err(|_| state_unavailable())?;
            oidc_pending.retain(|_, entry| entry.expires_at > now);
            oidc_pending.insert(
                state.to_string(),
                Entry {
                    value: pending,
                    expires_at: now.saturating_add(ttl_secs),
                },
            );
            Ok(())
        }

        async fn take_oidc_pending(
            &self,
            state: &str,
        ) -> Result<Option<OidcPendingRecord>, AuthError> {
            let mut oidc_pending = self.oidc_pending.write().map_err(|_| state_unavailable())?;
            let now = unix_now();
            match oidc_pending.remove(state) {
                Some(entry) if entry.expires_at > now => Ok(Some(entry.value)),
                _ => Ok(None),
            }
        }

        async fn is_oidc_state_used(&self, state: &str) -> Result<bool, AuthError> {
            let mut used = self
                .used_oidc_states
                .write()
                .map_err(|_| state_unavailable())?;
            let now = unix_now();
            used.retain(|_, expires_at| *expires_at > now);
            Ok(used.contains_key(state))
        }

        async fn mark_oidc_state_used(&self, state: &str, ttl_secs: u64) -> Result<(), AuthError> {
            self.used_oidc_states
                .write()
                .map_err(|_| state_unavailable())?
                .insert(state.to_string(), unix_now().saturating_add(ttl_secs));
            Ok(())
        }

        async fn put_wallet_challenge(
            &self,
            challenge_id: &str,
            challenge: WalletChallengeRecord,
            ttl_secs: u64,
        ) -> Result<(), AuthError> {
            let now = unix_now();
            let mut challenges = self
                .wallet_challenges
                .write()
                .map_err(|_| state_unavailable())?;
            challenges.retain(|_, entry| entry.expires_at > now);
            challenges.insert(
                challenge_id.to_string(),
                Entry {
                    value: challenge,
                    expires_at: now.saturating_add(ttl_secs),
                },
            );
            Ok(())
        }

        async fn take_wallet_challenge(
            &self,
            challenge_id: &str,
        ) -> Result<Option<WalletChallengeRecord>, AuthError> {
            let mut challenges = self
                .wallet_challenges
                .write()
                .map_err(|_| state_unavailable())?;
            let now = unix_now();
            match challenges.remove(challenge_id) {
                Some(entry) if entry.expires_at > now => Ok(Some(entry.value)),
                _ => Ok(None),
            }
        }

        async fn is_wallet_challenge_used(&self, challenge_id: &str) -> Result<bool, AuthError> {
            let mut used = self
                .used_wallet_challenges
                .write()
                .map_err(|_| state_unavailable())?;
            let now = unix_now();
            used.retain(|_, expires_at| *expires_at > now);
            Ok(used.contains_key(challenge_id))
        }

        async fn mark_wallet_challenge_used(
            &self,
            challenge_id: &str,
            ttl_secs: u64,
        ) -> Result<(), AuthError> {
            self.used_wallet_challenges
                .write()
                .map_err(|_| state_unavailable())?
                .insert(
                    challenge_id.to_string(),
                    unix_now().saturating_add(ttl_secs),
                );
            Ok(())
        }
    }
}

pub use in_memory::InMemoryEphemeralStore;

pub mod redis_store {
    use redis::AsyncCommands;

    use super::*;

    fn ser_error(_: serde_json::Error) -> AuthError {
        AuthError::PolicyViolation("failed to encode ephemeral state".to_string())
    }
    fn redis_error(e: redis::RedisError) -> AuthError {
        AuthError::PolicyViolation(format!("redis error: {e}"))
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
    impl EphemeralStore for RedisEphemeralStore {
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

        async fn put_oidc_pending(
            &self,
            state: &str,
            pending: OidcPendingRecord,
            ttl_secs: u64,
        ) -> Result<(), AuthError> {
            self.set_json(&format!("oidc_pending:{state}"), &pending, ttl_secs)
                .await
        }

        async fn take_oidc_pending(
            &self,
            state: &str,
        ) -> Result<Option<OidcPendingRecord>, AuthError> {
            self.take_json(&format!("oidc_pending:{state}")).await
        }

        async fn is_oidc_state_used(&self, state: &str) -> Result<bool, AuthError> {
            self.exists(&format!("oidc_used:{state}")).await
        }

        async fn mark_oidc_state_used(&self, state: &str, ttl_secs: u64) -> Result<(), AuthError> {
            self.mark_used(&format!("oidc_used:{state}"), ttl_secs)
                .await
        }

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
}

pub use redis_store::RedisEphemeralStore;
