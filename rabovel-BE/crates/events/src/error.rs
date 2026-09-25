#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("failed to serialize event payload: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("event envelope had type \"{found}\", expected \"{expected}\"")]
    TopicMismatch { expected: String, found: String },
}
