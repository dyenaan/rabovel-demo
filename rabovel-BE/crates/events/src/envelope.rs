use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::EventError;

/// Implemented by every payload type in `crate::schema`. `TOPIC` and
/// `EVENT_TYPE` are the single source of truth a producer/consumer pair
/// agrees on — a consumer decoding an envelope checks `event_type` against
/// this constant and rejects a mismatch rather than trusting the topic alone.
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync + 'static {
    const EVENT_TYPE: &'static str;
    const TOPIC: &'static str;
}

/// The wire format every event is wrapped in. Downstream crates
/// (reconciliation, settlement-orchestrator, tokenization-orchestrator)
/// only ever depend on this shape plus the payload structs in
/// `crate::schema` — never on application or domain-internal types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub event_id: String,
    pub event_type: String,
    pub occurred_at: u64,
    pub producer: String,
    pub schema_version: u32,
    pub payload: T,
}

impl<T: DomainEvent> EventEnvelope<T> {
    pub fn new(payload: T, producer: impl Into<String>) -> Self {
        Self {
            event_id: random_event_id(),
            event_type: T::EVENT_TYPE.to_string(),
            occurred_at: now_unix(),
            producer: producer.into(),
            schema_version: 1,
            payload,
        }
    }

    pub fn to_json(&self) -> Result<Vec<u8>, EventError> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Decodes a raw Kafka record value into `EventEnvelope<T>`, rejecting a
    /// payload whose `event_type` does not match `T::EVENT_TYPE` — this is
    /// what lets multiple event types share transport/consumer plumbing
    /// without a consumer silently misinterpreting the wrong shape.
    ///
    /// `event_type` is checked against a loosely-typed parse of the envelope
    /// *before* the payload is deserialized into `T`, so a genuine type
    /// mismatch is always reported as `TopicMismatch` — never masked by a
    /// field-shape error from trying to force an unrelated payload into `T`.
    pub fn from_json(bytes: &[u8]) -> Result<Self, EventError> {
        #[derive(Deserialize)]
        struct RawEnvelope {
            event_id: String,
            event_type: String,
            occurred_at: u64,
            producer: String,
            schema_version: u32,
            payload: serde_json::Value,
        }

        let raw: RawEnvelope = serde_json::from_slice(bytes)?;
        if raw.event_type != T::EVENT_TYPE {
            return Err(EventError::TopicMismatch {
                expected: T::EVENT_TYPE.to_string(),
                found: raw.event_type,
            });
        }
        let payload: T = serde_json::from_value(raw.payload)?;
        Ok(Self {
            event_id: raw.event_id,
            event_type: raw.event_type,
            occurred_at: raw.occurred_at,
            producer: raw.producer,
            schema_version: raw.schema_version,
            payload,
        })
    }
}

fn random_event_id() -> String {
    let mut bytes = [0u8; 16];
    // getrandom failure here would mean the OS RNG is unavailable, which is
    // already fatal for every other part of this platform (session tokens,
    // wallet challenges); falling back to a fixed id would be worse than
    // panicking, so this is intentionally infallible from the caller's view.
    getrandom::getrandom(&mut bytes).expect("OS random source unavailable");
    hex::encode(bytes)
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
