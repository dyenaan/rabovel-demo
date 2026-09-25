use std::sync::Arc;

use futures::StreamExt;
use rskafka::client::consumer::{StartOffset, StreamConsumer, StreamConsumerBuilder};
use rskafka::client::partition::UnknownTopicHandling;
use rskafka::client::ClientBuilder;

use events::{DomainEvent, EventEnvelope};

use super::MessagingError;

/// A single-topic, single-partition (0) consumer. rskafka has no built-in
/// consumer-group/offset-tracking support (by design — see its README), so
/// this always starts from `StartOffset::Earliest`; a service that needs to
/// resume from where it left off across restarts must persist its own last
/// processed offset and pass `StartOffset::At(offset + 1)` to `connect`.
pub struct TopicConsumer {
    inner: StreamConsumer,
}

impl TopicConsumer {
    pub async fn connect(
        bootstrap_servers: Vec<String>,
        topic: &str,
        start_offset: StartOffset,
    ) -> Result<Self, MessagingError> {
        let client = ClientBuilder::new(bootstrap_servers)
            .build()
            .await
            .map_err(|e| MessagingError::Kafka(e.to_string()))?;
        let partition_client = Arc::new(
            client
                .partition_client(topic, 0, UnknownTopicHandling::Retry)
                .await
                .map_err(|e| MessagingError::Kafka(e.to_string()))?,
        );
        let inner = StreamConsumerBuilder::new(partition_client, start_offset)
            .with_max_wait_ms(1_000)
            .build();
        Ok(Self { inner })
    }

    /// Awaits the next record and decodes it as `E`. Returns `None` once the
    /// underlying stream terminates (only happens after a fetch error, per
    /// rskafka's contract).
    pub async fn next_event<E: DomainEvent>(
        &mut self,
    ) -> Option<Result<EventEnvelope<E>, MessagingError>> {
        let item = self.inner.next().await?;
        Some(match item {
            Ok((record_and_offset, _high_watermark)) => match record_and_offset.record.value {
                Some(bytes) => EventEnvelope::<E>::from_json(&bytes).map_err(Into::into),
                None => Err(MessagingError::Kafka(
                    "received a record with no value".to_string(),
                )),
            },
            Err(err) => Err(MessagingError::Kafka(err.to_string())),
        })
    }
}

/// Default broker list used when `KAFKA_BOOTSTRAP_SERVERS` is unset.
pub const DEFAULT_BOOTSTRAP_SERVERS: &str = "localhost:9092";

/// Parses a comma-separated broker list, trimming whitespace and dropping
/// empty entries so `"a:9092, b:9092,"` yields two servers rather than three.
pub fn parse_bootstrap_servers(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Reads `KAFKA_BOOTSTRAP_SERVERS` from the environment, falling back to
/// [`DEFAULT_BOOTSTRAP_SERVERS`]. Shared by every consumer binary so the
/// env contract is defined in exactly one place.
pub fn bootstrap_servers_from_env() -> Vec<String> {
    let raw = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
        .unwrap_or_else(|_| DEFAULT_BOOTSTRAP_SERVERS.to_string());
    let servers = parse_bootstrap_servers(&raw);
    if servers.is_empty() {
        parse_bootstrap_servers(DEFAULT_BOOTSTRAP_SERVERS)
    } else {
        servers
    }
}

/// Connects a [`TopicConsumer`] for `topic` and logs every decoded event of
/// type `T` as it arrives. Never panics: a connect failure or a per-record
/// decode error is logged and does not crash the process.
///
/// This is the placeholder consumer loop each worker binary runs until its
/// real business logic lands. Replace the `info!` body with a handler when
/// that happens; the connect/decode/error plumbing stays the same.
pub async fn log_events<T>(bootstrap_servers: Vec<String>, topic: &'static str)
where
    T: DomainEvent + std::fmt::Debug,
{
    let mut consumer =
        match TopicConsumer::connect(bootstrap_servers, topic, StartOffset::Earliest).await {
            Ok(consumer) => consumer,
            Err(err) => {
                tracing::error!(topic, error = %err, "failed to connect topic consumer");
                return;
            }
        };

    loop {
        match consumer.next_event::<T>().await {
            Some(Ok(envelope)) => {
                tracing::info!(
                    topic,
                    event_id = %envelope.event_id,
                    event_type = %envelope.event_type,
                    payload = ?envelope.payload,
                    "received event"
                );
            }
            Some(Err(err)) => {
                tracing::warn!(topic, error = %err, "failed to decode event, skipping");
            }
            None => {
                tracing::info!(topic, "consumer stream ended");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_trims_whitespace_and_drops_empty_entries() {
        assert_eq!(
            parse_bootstrap_servers(" kafka:9092 , other:9092,, "),
            vec!["kafka:9092".to_string(), "other:9092".to_string()]
        );
    }

    #[test]
    fn parse_single_server() {
        assert_eq!(
            parse_bootstrap_servers("localhost:9092"),
            vec!["localhost:9092".to_string()]
        );
    }

    #[test]
    fn parse_empty_yields_nothing() {
        assert!(parse_bootstrap_servers("").is_empty());
        assert!(parse_bootstrap_servers(" , ").is_empty());
    }
}
