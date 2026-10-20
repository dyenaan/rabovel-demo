use identity::BrokerageOnboardingStatus;
use identity::UserRole;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentUserResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<UserRole>,
    pub onboarding_status: BrokerageOnboardingStatus,
}
