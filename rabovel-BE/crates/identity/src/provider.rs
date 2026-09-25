use async_trait::async_trait;
use domain::auth::{AuthError, VerifiedGoogleIdentity};

#[derive(Debug, Clone)]
pub struct OidcPending {
    pub nonce: String,
    pub pkce_verifier: String,
}

#[derive(Debug, Clone)]
pub struct OidcStart {
    pub authorization_url: String,
    pub state: String,
    pub pending: OidcPending,
}

#[async_trait]
pub trait GoogleIdentityProvider: Send + Sync {
    fn begin(&self) -> Result<OidcStart, AuthError>;
    async fn exchange(
        &self,
        code: &str,
        pending: &OidcPending,
    ) -> Result<VerifiedGoogleIdentity, AuthError>;
}

pub trait WalletProofVerifier: Send + Sync {
    fn verify(&self, message: &str, signature: &str) -> Result<String, AuthError>;
}
