use serde::{Deserialize, Serialize};

use crate::{envelope::DomainEvent, topics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizationRequested {
    pub request_id: String,
    pub user_id: String,
    pub asset_ref: String,
    pub amount: String,
    pub requested_at: u64,
}

impl DomainEvent for TokenizationRequested {
    const EVENT_TYPE: &'static str = "tokenization.requested";
    const TOPIC: &'static str = topics::TOKENIZATION_REQUESTED_V1;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizationCompleted {
    pub request_id: String,
    pub user_id: String,
    pub token_ref: String,
    pub completed_at: u64,
}

impl DomainEvent for TokenizationCompleted {
    const EVENT_TYPE: &'static str = "tokenization.completed";
    const TOPIC: &'static str = topics::TOKENIZATION_COMPLETED_V1;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizationFailed {
    pub request_id: String,
    pub user_id: String,
    pub reason: String,
    pub failed_at: u64,
}

impl DomainEvent for TokenizationFailed {
    const EVENT_TYPE: &'static str = "tokenization.failed";
    const TOPIC: &'static str = topics::TOKENIZATION_FAILED_V1;
}
