#[derive(Debug, thiserror::Error)]
pub enum MessagingError {
    #[error(transparent)]
    Event(#[from] events::EventError),
    #[error("kafka error: {0}")]
    Kafka(String),
    #[error("event source is unavailable: {0}")]
    Unavailable(String),
}
