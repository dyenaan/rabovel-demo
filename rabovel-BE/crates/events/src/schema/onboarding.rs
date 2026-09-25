use serde::{Deserialize, Serialize};

use crate::{envelope::DomainEvent, topics};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStatus {
    Approved,
    RequiresReview,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnboardingStatusChanged {
    pub user_id: String,
    pub previous_status: OnboardingStatus,
    pub new_status: OnboardingStatus,
    pub occurred_at: u64,
}

impl DomainEvent for OnboardingStatusChanged {
    const EVENT_TYPE: &'static str = "onboarding.status_changed";
    const TOPIC: &'static str = topics::ONBOARDING_STATUS_CHANGED_V1;
}
