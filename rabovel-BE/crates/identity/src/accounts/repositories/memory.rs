use super::super::models::*;
use super::super::ports::AccountRepository;
use crate::AuthError;
use crate::AuthPrincipal;
use crate::IdentityProfile;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("identity repository unavailable".into())
}
#[derive(Default)]
pub struct InMemoryAccounts {
    users: RwLock<HashMap<String, UserRecord>>,
    password_credentials: RwLock<HashMap<String, PasswordCredential>>,
}
#[async_trait]
impl AccountRepository for InMemoryAccounts {
    async fn create_password_user(
        &self,
        credential: PasswordCredential,
        profile: IdentityProfile,
        principal: AuthPrincipal,
    ) -> Result<UserRecord, AuthError> {
        let mut credentials = self
            .password_credentials
            .write()
            .map_err(|_| state_unavailable())?;
        if credentials.contains_key(&credential.email) {
            return Err(AuthError::PolicyViolation(
                "an account already exists for this email".to_string(),
            ));
        }
        let record = UserRecord { profile, principal };
        self.users
            .write()
            .map_err(|_| state_unavailable())?
            .insert(credential.user_id.clone(), record.clone());
        credentials.insert(credential.email.clone(), credential);
        Ok(record)
    }
    async fn get_password_credential(
        &self,
        email: &str,
    ) -> Result<Option<PasswordCredential>, AuthError> {
        Ok(self
            .password_credentials
            .read()
            .map_err(|_| state_unavailable())?
            .get(email)
            .cloned())
    }
    async fn get_password_credential_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Option<PasswordCredential>, AuthError> {
        Ok(self
            .password_credentials
            .read()
            .map_err(|_| state_unavailable())?
            .values()
            .find(|credential| credential.user_id == user_id)
            .cloned())
    }
}
#[async_trait::async_trait]
impl super::super::ports::AccountQueries for InMemoryAccounts {
    async fn get_user(&self, user_id: &str) -> Result<Option<UserRecord>, AuthError> {
        Ok(self
            .users
            .read()
            .map_err(|_| state_unavailable())?
            .get(user_id)
            .cloned())
    }
}

impl InMemoryAccounts {
    /// Seed an isolated in-memory store with trusted records, useful for composition/tests.
    pub fn with_account(
        credential: PasswordCredential,
        profile: IdentityProfile,
        principal: AuthPrincipal,
    ) -> Self {
        let key = credential.email.clone();
        let user = UserRecord { profile, principal };
        Self {
            users: RwLock::new(HashMap::from([(credential.user_id.clone(), user)])),
            password_credentials: RwLock::new(HashMap::from([(key, credential)])),
        }
    }
}

#[async_trait::async_trait]
impl super::super::ports::ProfileUpdates for InMemoryAccounts {
    async fn save_profile(&self, profile: IdentityProfile) -> Result<(), AuthError> {
        let mut users = self.users.write().map_err(|_| state_unavailable())?;
        let record = users
            .get_mut(profile.user_id())
            .ok_or(AuthError::Unauthorized)?;
        record.profile = profile;
        Ok(())
    }
}
#[cfg(test)]
mod profile_tests {
    use super::*;
    use crate::accounts::ports::{AccountQueries, ProfileUpdates};
    use crate::{BrokerageAuthService, BrokerageSecurityPolicy, UserRole, WalletAction};
    use std::collections::HashSet;
    #[tokio::test]
    async fn compliance_updates_preserve_roles_permissions_and_credentials() {
        let profile = BrokerageAuthService::new(BrokerageSecurityPolicy::for_testing())
            .profile_from_email_password("usr_one", "issuer@example.com")
            .unwrap();
        let principal = AuthPrincipal::new(
            "usr_one",
            HashSet::from([UserRole::Issuer]),
            HashSet::from([WalletAction::ViewPortfolio]),
        );
        let store = InMemoryAccounts::with_account(
            PasswordCredential {
                user_id: "usr_one".into(),
                email: "issuer@example.com".into(),
                display_name: "Issuer".into(),
                password_hash: "unchanged-hash".into(),
            },
            profile.clone(),
            principal,
        );
        let mut updated = profile;
        updated.accept_terms("new-terms").unwrap();
        store.save_profile(updated).await.unwrap();
        let account = store.get_user("usr_one").await.unwrap().unwrap();
        assert!(account.principal.has_role(UserRole::Issuer));
        assert!(!account.principal.has_role(UserRole::Trader));
        assert_eq!(account.profile.terms_version(), Some("new-terms"));
        assert_eq!(
            store
                .get_password_credential("issuer@example.com")
                .await
                .unwrap()
                .unwrap()
                .password_hash,
            "unchanged-hash"
        );
    }
}
