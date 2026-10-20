use crate::{DomainEvent, EventEnvelope};
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    Event(#[from] crate::EventError),
    #[error("event transport error: {0}")]
    Transport(String),
    #[error("event source is unavailable: {0}")]
    Unavailable(String),
}

/// Publishes an already-serialized envelope. This is intentionally
/// non-generic (rather than a generic `publish<E>`) so `EventProducer` stays
/// object-safe and can be stored as `Arc<dyn EventProducer>` inside a
/// long-lived service like the BFF's `GatewayState`. Callers use the
/// free function [`publish`] below, which builds the envelope for you.
#[async_trait]
pub trait EventProducer: Send + Sync {
    async fn publish_raw(
        &self,
        topic: &str,
        key: &str,
        payload: Vec<u8>,
    ) -> Result<(), PublishError>;
}
/// Wraps `event` in an [`EventEnvelope`], serializes it, and hands it to
/// `producer` under `E::TOPIC`.
pub async fn publish<E: DomainEvent>(
    producer: &dyn EventProducer,
    event: E,
    produced_by: &str,
) -> Result<(), PublishError> {
    let envelope = EventEnvelope::new(event, produced_by);
    let key = envelope.event_id.clone();
    let bytes = envelope.to_json()?;
    producer.publish_raw(E::TOPIC, &key, bytes).await
}
