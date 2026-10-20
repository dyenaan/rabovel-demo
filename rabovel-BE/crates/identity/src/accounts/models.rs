use crate::AuthPrincipal;
use crate::IdentityProfile;

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub profile: IdentityProfile,
    pub principal: AuthPrincipal,
}
#[derive(Debug, Clone)]
pub struct PasswordCredential {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
}
