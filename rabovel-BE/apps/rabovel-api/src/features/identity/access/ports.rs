use std::sync::Arc;
pub(crate) trait RateLimit {
    fn rate_limiter(&self) -> &Arc<dyn identity::access::rate_limiter::RateLimiter>;
}
pub(crate) trait AccessPolicy {
    fn auth_service(&self) -> &Arc<identity::BrokerageAuthService>;
}
