use identity::BrokerageOnboardingStatus;
use identity::IdentityProfile;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct OnboardingStatusResponse {
    pub profile: IdentityProfile,
    pub status: BrokerageOnboardingStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TermsAcceptanceRequest {
    pub version: String,
}
