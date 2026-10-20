//! Kafka implementations of the event contract port.
pub mod consumer;
pub mod producer;
pub use events::{publish, EventProducer, PublishError as MessagingError};
pub use producer::{InMemoryEventProducer, KafkaEventProducer, NoopEventProducer};
