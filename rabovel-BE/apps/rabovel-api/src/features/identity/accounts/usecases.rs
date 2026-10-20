use crate::app::constants::SESSION_TTL_SECONDS;
use crate::app::errors::ApplicationError;
use crate::app::support::random_hex;
use crate::features::identity::accounts::dto::AuthSessionResponse;
use crate::features::identity::accounts::dto::PasswordLoginRequest;
use crate::features::identity::accounts::dto::PasswordRegisterRequest;
use crate::features::identity::accounts::dto::SignupAccountType;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::PasswordHash;
use argon2::password_hash::PasswordHasher;
use argon2::password_hash::PasswordVerifier;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use identity::AuthError;
use identity::AuthPrincipal;
use identity::PasswordCredential;
use identity::UserRole;
use identity::WalletAction;
use std::collections::HashSet;

pub(crate) async fn password_register(
    state: &impl Ports,
    request: PasswordRegisterRequest,
) -> Result<AuthSessionResponse, ApplicationError> {
    let email = request.email.trim().to_ascii_lowercase();
    let display_name = request.display_name.trim().to_string();
    if !email.contains('@') || display_name.len() < 2 || request.password.len() < 8 {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "a valid name, email, and password of at least 8 characters are required".to_string(),
        )));
    }
    let user_id = format!("usr_{}", random_hex(16).map_err(ApplicationError::from)?);
    let mut profile = state
        .auth_service()
        .profile_from_email_password(user_id.clone(), email.clone())
        .map_err(ApplicationError::from)?;
    profile
        .accept_terms("demo-v1")
        .map_err(ApplicationError::from)?;
    let role = match request.account_type {
        SignupAccountType::Investor => UserRole::Trader,
        SignupAccountType::Issuer => UserRole::Issuer,
    };
    let mut permissions = HashSet::from([WalletAction::ViewPortfolio]);
    if role == UserRole::Trader {
        permissions.insert(WalletAction::TradeSpot);
    }
    let principal = AuthPrincipal::new(user_id.clone(), HashSet::from([role]), permissions);
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(|_| {
            ApplicationError::from(AuthError::PolicyViolation(
                "password hashing failed".to_string(),
            ))
        })?
        .to_string();
    let record = state
        .accounts()
        .create_password_user(
            PasswordCredential {
                user_id: user_id.clone(),
                email,
                display_name,
                password_hash,
            },
            profile,
            principal,
        )
        .await
        .map_err(ApplicationError::from)?;
    let status = state.auth_service().onboarding_status(&record.profile);
    let access_token =
        crate::features::identity::sessions::usecases::issue_session(state, &user_id, None)
            .await
            .map_err(ApplicationError::from)?;
    Ok(AuthSessionResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: SESSION_TTL_SECONDS,
        profile: record.profile,
        status,
    })
}

pub(crate) async fn password_login(
    state: &impl Ports,
    request: PasswordLoginRequest,
) -> Result<AuthSessionResponse, ApplicationError> {
    let email = request.email.trim().to_ascii_lowercase();
    password_session(state, &email, &request.password)
        .await
        .map_err(ApplicationError::from)
}

pub(crate) async fn password_session(
    state: &impl Ports,
    email: &str,
    password: &str,
) -> Result<AuthSessionResponse, AuthError> {
    let credential = state
        .accounts()
        .get_password_credential(email)
        .await?
        .ok_or(AuthError::Unauthorized)?;
    let hash = PasswordHash::new(&credential.password_hash).map_err(|_| AuthError::Unauthorized)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .map_err(|_| AuthError::Unauthorized)?;
    let record = state
        .accounts()
        .get_user(&credential.user_id)
        .await?
        .ok_or(AuthError::Unauthorized)?;
    let status = state.auth_service().onboarding_status(&record.profile);
    let access_token = crate::features::identity::sessions::usecases::issue_session(
        state,
        &credential.user_id,
        None,
    )
    .await?;
    Ok(AuthSessionResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: SESSION_TTL_SECONDS,
        profile: record.profile,
        status,
    })
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
