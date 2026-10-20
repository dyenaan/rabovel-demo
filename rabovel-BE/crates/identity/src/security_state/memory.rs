use crate::sessions::models::SessionRecord;
use crate::sessions::ports::SessionRepository;
use crate::wallets::linking::models::WalletChallengeRecord;
use crate::wallets::linking::ports::ChallengeRepository;
use crate::AuthError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

fn state_unavailable() -> AuthError {
    AuthError::ServiceUnavailable
}

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
    wallet_challenges: RwLock<HashMap<String, Entry<WalletChallengeRecord>>>,
    used_wallet_challenges: RwLock<HashMap<String, u64>>,
}

#[async_trait]
impl SessionRepository for InMemoryEphemeralStore {
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
}

#[async_trait]
impl ChallengeRepository for InMemoryEphemeralStore {
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
