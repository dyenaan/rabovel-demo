pub(crate) const WALLET_CHALLENGE_TTL_SECONDS: u64 = 300;

pub(crate) const SESSION_TTL_SECONDS: u64 = 900;

/// Applied per authenticated session on every financial-action route.
pub(crate) const SESSION_RATE_LIMIT: (u32, u64) = (10, 60);
/// Applied per client IP supplied by the authenticated gateway, or the
/// socket peer in direct local development, across application routes.
pub(crate) const IP_RATE_LIMIT: (u32, u64) = (60, 60);
