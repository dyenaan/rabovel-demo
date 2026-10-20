use super::models::*;
use crate::AuthError;
use crate::AuthPrincipal;
use crate::IdentityProfile;
use async_trait::async_trait;

#[async_trait]
pub trait AccountRepository: ProfileUpdates + AccountQueries + Send + Sync {
    async fn create_password_user(
        &self,
        credential: PasswordCredential,
        profile: IdentityProfile,
        principal: AuthPrincipal,
    ) -> Result<UserRecord, AuthError>;
    async fn get_password_credential(
        &self,
        email: &str,
    ) -> Result<Option<PasswordCredential>, AuthError>;
    async fn get_password_credential_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Option<PasswordCredential>, AuthError>;
}

#[async_trait::async_trait]
pub trait AccountQueries: Send + Sync {
    async fn get_user(&self, user_id: &str) -> Result<Option<UserRecord>, AuthError>;
}

#[async_trait::async_trait]
pub trait ProfileUpdates: Send + Sync {
    async fn save_profile(&self, profile: IdentityProfile) -> Result<(), AuthError>;
}
