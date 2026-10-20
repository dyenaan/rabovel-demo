use identity::AuthError;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Encodes a `serde`-serializable unit-variant enum as its bare string form
/// (the same string domain's `#[serde(rename_all = "snake_case")]` already
/// produces), for embedding in an event payload's `String` fields.
pub(crate) fn serde_str<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

pub(crate) fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(crate) fn random_hex(bytes: usize) -> Result<String, AuthError> {
    let mut value = vec![0_u8; bytes];
    getrandom::getrandom(&mut value).map_err(|_| state_unavailable())?;
    Ok(hex::encode(value))
}

pub(crate) fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub(crate) fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("authentication state is unavailable".to_string())
}
