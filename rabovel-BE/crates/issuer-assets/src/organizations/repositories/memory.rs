use super::super::models::*;
use super::super::ports::OrganizationRepository;
use async_trait::async_trait;
use identity::AuthError;
use std::collections::HashMap;
use std::sync::RwLock;

fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("identity repository unavailable".into())
}
#[derive(Default)]
pub struct InMemoryOrganizations {
    issuer_organizations: RwLock<HashMap<String, IssuerOrganization>>,
}
#[async_trait]
impl OrganizationRepository for InMemoryOrganizations {
    async fn save_issuer_organization(
        &self,
        organization: IssuerOrganization,
    ) -> Result<(), AuthError> {
        self.issuer_organizations
            .write()
            .map_err(|_| state_unavailable())?
            .insert(organization.user_id.clone(), organization);
        Ok(())
    }
    async fn get_issuer_organization(
        &self,
        user_id: &str,
    ) -> Result<Option<IssuerOrganization>, AuthError> {
        Ok(self
            .issuer_organizations
            .read()
            .map_err(|_| state_unavailable())?
            .get(user_id)
            .cloned())
    }
}
