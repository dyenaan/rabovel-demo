//! Shared TTL implementation for identity sessions and ownership challenges.

pub mod memory;
pub mod redis;
pub trait EphemeralStore:
    crate::sessions::ports::SessionRepository + crate::wallets::linking::ports::ChallengeRepository
{
}
impl<
        T: crate::sessions::ports::SessionRepository
            + crate::wallets::linking::ports::ChallengeRepository
            + ?Sized,
    > EphemeralStore for T
{
}
pub use memory::InMemoryEphemeralStore;
pub use redis::RedisEphemeralStore;
