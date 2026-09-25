//! Single source of truth for Kafka topic names. Every payload type in
//! `crate::schema` points back at one of these via `DomainEvent::TOPIC`.
//! Topics are versioned in-name (`.v1`) so a breaking schema change ships as
//! a new topic rather than an in-place mutation existing consumers silently
//! misread.

pub const ONBOARDING_STATUS_CHANGED_V1: &str = "rabovel.onboarding.status-changed.v1";
pub const WALLET_LINKED_V1: &str = "rabovel.wallet.linked.v1";
pub const KYC_CASE_SUBMITTED_V1: &str = "rabovel.kyc.case-submitted.v1";
pub const KYC_VERDICT_RECEIVED_V1: &str = "rabovel.kyc.verdict-received.v1";
pub const TRADING_ORDER_ACCEPTED_V1: &str = "rabovel.trading.order-accepted.v1";
pub const TRADING_TRADE_EXECUTED_V1: &str = "rabovel.trading.trade-executed.v1";
pub const PORTFOLIO_CHANGED_V1: &str = "rabovel.portfolio.changed.v1";
pub const TOKENIZATION_REQUESTED_V1: &str = "rabovel.tokenization.requested.v1";
pub const TOKENIZATION_COMPLETED_V1: &str = "rabovel.tokenization.completed.v1";
pub const TOKENIZATION_FAILED_V1: &str = "rabovel.tokenization.failed.v1";

/// Every topic this platform defines, for provisioning (topic auto-creation
/// is on in the local Kafka container, but a production cluster would use
/// this list to pre-create topics with real partition/replication settings).
pub const ALL_TOPICS: &[&str] = &[
    ONBOARDING_STATUS_CHANGED_V1,
    WALLET_LINKED_V1,
    KYC_CASE_SUBMITTED_V1,
    KYC_VERDICT_RECEIVED_V1,
    TRADING_ORDER_ACCEPTED_V1,
    TRADING_TRADE_EXECUTED_V1,
    PORTFOLIO_CHANGED_V1,
    TOKENIZATION_REQUESTED_V1,
    TOKENIZATION_COMPLETED_V1,
    TOKENIZATION_FAILED_V1,
];
