use identity::BrokerageOnboardingStatus;
use identity::IdentityProfile;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

/// Public signup can never select an operational or administrative role.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SignupAccountType {
    Investor,
    Issuer,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PasswordRegisterRequest {
    pub account_type: SignupAccountType,
    pub display_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PasswordLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthSessionResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub profile: IdentityProfile,
    pub status: BrokerageOnboardingStatus,
}
