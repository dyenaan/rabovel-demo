use events::EventProducer;

use async_trait::async_trait;

use super::MessagingError;

/// Publishes nothing. Used where no broker is configured, e.g. tests or a
/// gateway started via `GatewayState::disabled()`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopEventProducer;

#[async_trait]
impl EventProducer for NoopEventProducer {
    async fn publish_raw(
        &self,
        _topic: &str,
        _key: &str,
        _payload: Vec<u8>,
    ) -> Result<(), MessagingError> {
        Ok(())
    }
}

/// Records every publish in memory. Used by tests that need to assert an
/// event was emitted without a running broker.
#[derive(Debug, Default)]
pub struct InMemoryEventProducer {
    published: tokio::sync::Mutex<Vec<(String, String, Vec<u8>)>>,
}

impl InMemoryEventProducer {
    pub async fn published(&self) -> Vec<(String, String, Vec<u8>)> {
        self.published.lock().await.clone()
    }
}

#[async_trait]
impl EventProducer for InMemoryEventProducer {
    async fn publish_raw(
        &self,
        topic: &str,
        key: &str,
        payload: Vec<u8>,
    ) -> Result<(), MessagingError> {
        self.published
            .lock()
            .await
            .push((topic.to_string(), key.to_string(), payload));
        Ok(())
    }
}

mod kafka_producer {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use rskafka::chrono::{DateTime, Utc};
    use rskafka::client::partition::{Compression, PartitionClient, UnknownTopicHandling};
    use rskafka::client::{Client, ClientBuilder};
    use rskafka::record::Record;
    use tokio::sync::RwLock;

    use super::EventProducer;
    use super::MessagingError;

    /// A real Kafka-backed producer. One partition (0) per topic is used —
    /// this platform's topics are low-throughput integration-event streams,
    /// not high-volume data streams, so a single partition per topic keeps
    /// per-topic ordering simple for consumers. A production deployment
    /// wanting more partitions would extend `publish_raw` to pick a
    /// partition (e.g. hash of `key`) instead of always using 0.
    pub struct KafkaEventProducer {
        client: Client,
        partitions: RwLock<BTreeMap<String, Arc<PartitionClient>>>,
    }

    impl KafkaEventProducer {
        pub async fn connect(bootstrap_servers: Vec<String>) -> Result<Self, MessagingError> {
            let client = ClientBuilder::new(bootstrap_servers)
                .build()
                .await
                .map_err(|e| MessagingError::Transport(e.to_string()))?;
            Ok(Self {
                client,
                partitions: RwLock::new(BTreeMap::new()),
            })
        }

        async fn partition_client(
            &self,
            topic: &str,
        ) -> Result<Arc<PartitionClient>, MessagingError> {
            if let Some(existing) = self.partitions.read().await.get(topic) {
                return Ok(Arc::clone(existing));
            }
            let mut partitions = self.partitions.write().await;
            if let Some(existing) = partitions.get(topic) {
                return Ok(Arc::clone(existing));
            }
            let partition_client = Arc::new(
                self.client
                    .partition_client(topic, 0, UnknownTopicHandling::Retry)
                    .await
                    .map_err(|e| MessagingError::Transport(e.to_string()))?,
            );
            partitions.insert(topic.to_string(), Arc::clone(&partition_client));
            Ok(partition_client)
        }
    }

    #[async_trait]
    impl EventProducer for KafkaEventProducer {
        async fn publish_raw(
            &self,
            topic: &str,
            key: &str,
            payload: Vec<u8>,
        ) -> Result<(), MessagingError> {
            let partition_client = self.partition_client(topic).await?;
            let now_millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            let record = Record {
                key: Some(key.as_bytes().to_vec()),
                value: Some(payload),
                headers: BTreeMap::new(),
                timestamp: DateTime::<Utc>::from_timestamp_millis(now_millis)
                    .expect("system clock is within chrono's representable range"),
            };
            partition_client
                .produce(vec![record], Compression::NoCompression)
                .await
                .map_err(|e| MessagingError::Transport(e.to_string()))?;
            Ok(())
        }
    }
}

pub use kafka_producer::KafkaEventProducer;

#[cfg(test)]
mod tests {
    use super::*;
    use events::schema::onboarding::{OnboardingStatus, OnboardingStatusChanged};
    use events::{publish, DomainEvent};

    #[tokio::test]
    async fn in_memory_producer_records_publishes() {
        let producer = InMemoryEventProducer::default();
        publish(
            &producer,
            OnboardingStatusChanged {
                user_id: "user-1".to_string(),
                previous_status: OnboardingStatus::RequiresReview,
                new_status: OnboardingStatus::Approved,
                occurred_at: 1_800_000_000,
            },
            "order-gateway",
        )
        .await
        .unwrap();
        let published = producer.published().await;
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].0, OnboardingStatusChanged::TOPIC);
    }
}
