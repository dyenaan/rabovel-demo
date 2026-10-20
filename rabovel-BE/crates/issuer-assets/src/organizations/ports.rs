use super::models::*;
use async_trait::async_trait;
use identity::AuthError;

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn save_issuer_organization(
        &self,
        organization: IssuerOrganization,
    ) -> Result<(), AuthError>;
    async fn get_issuer_organization(
        &self,
        user_id: &str,
    ) -> Result<Option<IssuerOrganization>, AuthError>;
}
