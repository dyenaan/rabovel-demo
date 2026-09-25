//! Request rate limiting, layered on top of the WAF container that fronts
//! this service in production (see the `waf` docker-compose service) rather
//! than replacing it. This is the in-process half of that defense: a global
//! per-IP limit plus a stricter per-session limit on financial-action
//! routes.

use async_trait::async_trait;
use domain::auth::AuthError;

#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Checks (and, if allowed, consumes one unit of) `key`'s quota of
    /// `limit` requests per `window_secs`. Returns
    /// `Err(AuthError::RateLimited)` once the quota is exhausted.
    async fn check(&self, key: &str, limit: u32, window_secs: u64) -> Result<(), AuthError>;
}

pub mod in_memory {
    use std::collections::HashMap;
    use std::sync::RwLock;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    struct Window {
        started_at: u64,
        count: u32,
    }

    /// A fixed-window counter -- simple, and sufficient for tests/local dev
    /// where `GatewayState::disabled()` has no Redis to talk to. Production
    /// uses `RedisTokenBucketRateLimiter` instead.
    #[derive(Default)]
    pub struct InMemoryRateLimiter {
        windows: RwLock<HashMap<String, Window>>,
    }

    fn unix_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    #[async_trait]
    impl RateLimiter for InMemoryRateLimiter {
        async fn check(&self, key: &str, limit: u32, window_secs: u64) -> Result<(), AuthError> {
            let now = unix_now();
            let mut windows = self
                .windows
                .write()
                .map_err(|_| AuthError::PolicyViolation("rate limiter unavailable".to_string()))?;
            let window = windows.entry(key.to_string()).or_insert(Window {
                started_at: now,
                count: 0,
            });
            if now.saturating_sub(window.started_at) >= window_secs {
                window.started_at = now;
                window.count = 0;
            }
            if window.count >= limit {
                return Err(AuthError::RateLimited);
            }
            window.count += 1;
            Ok(())
        }
    }
}

pub use in_memory::InMemoryRateLimiter;

pub mod redis_limiter {
    use redis::Script;

    use super::*;

    /// Atomic refill-then-consume in one Redis round trip via a Lua script
    /// -- avoids the read-modify-write race a plain GET/SET pair would have
    /// under concurrent requests for the same key.
    const TOKEN_BUCKET_SCRIPT: &str = r#"
local bucket = redis.call('HMGET', KEYS[1], 'tokens', 'last_refill_ms')
local capacity = tonumber(ARGV[1])
local refill_per_ms = tonumber(ARGV[2])
local now_ms = tonumber(ARGV[3])
local ttl_secs = tonumber(ARGV[4])

local tokens = tonumber(bucket[1])
local last_refill_ms = tonumber(bucket[2])
if tokens == nil then
  tokens = capacity
  last_refill_ms = now_ms
end

local elapsed_ms = math.max(0, now_ms - last_refill_ms)
tokens = math.min(capacity, tokens + elapsed_ms * refill_per_ms)

local allowed = 0
if tokens >= 1 then
  tokens = tokens - 1
  allowed = 1
end

redis.call('HMSET', KEYS[1], 'tokens', tokens, 'last_refill_ms', now_ms)
redis.call('EXPIRE', KEYS[1], ttl_secs)

return allowed
"#;

    pub struct RedisTokenBucketRateLimiter {
        manager: redis::aio::ConnectionManager,
        script: Script,
    }

    impl RedisTokenBucketRateLimiter {
        pub async fn connect(redis_url: &str) -> Result<Self, String> {
            let client =
                redis::Client::open(redis_url).map_err(|e| format!("invalid REDIS_URL: {e}"))?;
            let manager = client
                .get_connection_manager()
                .await
                .map_err(|e| format!("failed to connect to Redis: {e}"))?;
            Ok(Self {
                manager,
                script: Script::new(TOKEN_BUCKET_SCRIPT),
            })
        }
    }

    #[async_trait]
    impl RateLimiter for RedisTokenBucketRateLimiter {
        async fn check(&self, key: &str, limit: u32, window_secs: u64) -> Result<(), AuthError> {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            let refill_per_ms = f64::from(limit) / (window_secs.max(1) as f64 * 1000.0);
            let mut conn = self.manager.clone();
            let allowed: i32 = self
                .script
                .key(format!("ratelimit:{key}"))
                .arg(limit)
                .arg(refill_per_ms)
                .arg(now_ms)
                .arg(window_secs.saturating_mul(2).max(1))
                .invoke_async(&mut conn)
                .await
                .map_err(|e| AuthError::PolicyViolation(format!("redis error: {e}")))?;
            if allowed == 1 {
                Ok(())
            } else {
                Err(AuthError::RateLimited)
            }
        }
    }
}

pub use redis_limiter::RedisTokenBucketRateLimiter;
