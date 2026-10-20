use crate::app::errors::ApplicationError;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::sessions::dto::CurrentUserResponse;
use identity::AuthError;

use crate::app::constants::SESSION_TTL_SECONDS;
use identity::sessions::models::SessionRecord;
use identity::UserRole;

use crate::app::support::hash_token;
use crate::app::support::random_hex;

pub(crate) async fn auth_me(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<CurrentUserResponse, ApplicationError> {
    let record = state
        .accounts()
        .get_user(&user.user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    let email = record.profile.email().to_string();
    let display_name = state
        .accounts()
        .get_password_credential_by_user_id(&user.user_id)
        .await
        .map_err(ApplicationError::from)?
        .map(|credential| credential.display_name)
        .unwrap_or_else(|| email.split('@').next().unwrap_or(&email).to_string());
    let mut roles: Vec<_> = record.principal.roles_iter().collect();
    roles.sort_by_key(|role| match role {
        UserRole::Admin => 0,
        UserRole::ComplianceOfficer => 1,
        UserRole::RiskOps => 2,
        UserRole::Issuer => 3,
        UserRole::Trader => 4,
    });
    Ok(CurrentUserResponse {
        user_id: user.user_id,
        email,
        display_name,
        roles,
        onboarding_status: state.auth_service().onboarding_status(&record.profile),
    })
}

pub(crate) async fn logout(
    state: &impl Ports,
    token: &str,
    _user: AuthenticatedUser,
) -> Result<(), ApplicationError> {
    state
        .sessions()
        .delete_session(&hash_token(token))
        .await
        .map_err(ApplicationError::from)?;
    Ok(())
}

pub(crate) async fn issue_session(
    state: &impl Ports,
    user_id: &str,
    mfa_verified_at: Option<u64>,
) -> Result<String, AuthError> {
    let token = random_hex(32)?;
    state
        .sessions()
        .create_session(
            &hash_token(&token),
            SessionRecord {
                user_id: user_id.to_string(),
                mfa_verified_at,
            },
            SESSION_TTL_SECONDS,
        )
        .await?;
    Ok(token)
}

/// Both an expired and a never-issued session are indistinguishable
/// once ephemeral state carries its own TTL (an expired entry simply
/// reads back as absent) -- this collapses what the original in-memory
/// version reported as two distinct errors (`SessionExpired` vs
/// `Unauthorized`) into one, but both already mapped to the same
/// `401 Unauthorized` HTTP status, so this is not a behavior change for
/// any caller.
pub(crate) async fn authenticate(
    state: &impl Ports,
    token: &str,
) -> Result<SessionRecord, AuthError> {
    state
        .sessions()
        .get_session(&hash_token(token))
        .await?
        .ok_or(AuthError::Unauthorized)
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountStore
    + crate::features::identity::access::ports::AccessPolicy
    + crate::features::identity::sessions::ports::SessionStore
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountStore
            + crate::features::identity::access::ports::AccessPolicy
            + crate::features::identity::sessions::ports::SessionStore
            + ?Sized,
    > Ports for T
{
}
