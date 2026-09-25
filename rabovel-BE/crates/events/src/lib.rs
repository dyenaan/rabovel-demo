//! Shared event contract for the Rabovel platform's Kafka integration bus.
//!
//! This crate deliberately has **zero dependency on `domain` or
//! application code — every payload type in [`schema`] is self-contained
//! plain data with its own local enums. `reconciliation`,
//! `settlement-orchestrator`, and `tokenization-orchestrator` depend only on
//! this crate (never on the gateway or its auth/identity trust-boundary
//! types) to consume the events they care about. That is the entire
//! plug-and-play mechanism between this platform and those crates.

pub mod envelope;
pub mod error;
pub mod schema;
pub mod topics;

pub use envelope::{DomainEvent, EventEnvelope};
pub use error::EventError;

#[cfg(test)]
mod tests {
    use super::*;
    use schema::kyc::KycCaseSubmitted;
    use schema::onboarding::{OnboardingStatus, OnboardingStatusChanged};

    #[test]
    fn envelope_round_trips_through_json() {
        let event = OnboardingStatusChanged {
            user_id: "user-1".to_string(),
            previous_status: OnboardingStatus::RequiresReview,
            new_status: OnboardingStatus::Approved,
            occurred_at: 1_800_000_000,
        };
        let envelope = EventEnvelope::new(event.clone(), "order-gateway");
        let bytes = envelope.to_json().unwrap();
        let decoded = EventEnvelope::<OnboardingStatusChanged>::from_json(&bytes).unwrap();
        assert_eq!(decoded.payload.user_id, event.user_id);
        assert_eq!(decoded.payload.new_status, OnboardingStatus::Approved);
        assert_eq!(decoded.event_type, OnboardingStatusChanged::EVENT_TYPE);
    }

    #[test]
    fn decoding_as_the_wrong_event_type_is_rejected() {
        let event = OnboardingStatusChanged {
            user_id: "user-1".to_string(),
            previous_status: OnboardingStatus::RequiresReview,
            new_status: OnboardingStatus::Approved,
            occurred_at: 1_800_000_000,
        };
        let bytes = EventEnvelope::new(event, "order-gateway")
            .to_json()
            .unwrap();
        let result = EventEnvelope::<KycCaseSubmitted>::from_json(&bytes);
        assert!(matches!(result, Err(EventError::TopicMismatch { .. })));
    }
}
