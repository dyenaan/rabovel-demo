use std::str::FromStr;

use async_trait::async_trait;
use domain::auth::{AuthError, VerifiedGoogleIdentity};
use identity::provider::{GoogleIdentityProvider, OidcPending, OidcStart, WalletProofVerifier};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope,
};
use siwe::Message;

type DiscoveredGoogleClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Debug)]
pub struct DisabledGoogleIdentityProvider;

#[async_trait]
impl GoogleIdentityProvider for DisabledGoogleIdentityProvider {
    fn begin(&self) -> Result<OidcStart, AuthError> {
        Err(AuthError::InvalidProvider)
    }

    async fn exchange(
        &self,
        _code: &str,
        _pending: &OidcPending,
    ) -> Result<VerifiedGoogleIdentity, AuthError> {
        Err(AuthError::InvalidProvider)
    }
}

pub struct GoogleOidcProvider {
    client: DiscoveredGoogleClient,
    http_client: reqwest::Client,
}

impl GoogleOidcProvider {
    pub async fn discover(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Result<Self, String> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "failed to construct the OIDC HTTP client".to_string())?;
        let issuer = IssuerUrl::new("https://accounts.google.com".to_string())
            .map_err(|_| "invalid Google issuer configuration".to_string())?;
        let metadata = CoreProviderMetadata::discover_async(issuer, &http_client)
            .await
            .map_err(|_| "Google OpenID discovery failed".to_string())?;
        let client = CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri)
                .map_err(|_| "GOOGLE_REDIRECT_URI is invalid".to_string())?,
        );
        Ok(Self {
            client,
            http_client,
        })
    }
}

#[async_trait]
impl GoogleIdentityProvider for GoogleOidcProvider {
    fn begin(&self) -> Result<OidcStart, AuthError> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorization_url, state, nonce) = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        Ok(OidcStart {
            authorization_url: authorization_url.to_string(),
            state: state.secret().to_string(),
            pending: OidcPending {
                nonce: nonce.secret().to_string(),
                pkce_verifier: pkce_verifier.secret().to_string(),
            },
        })
    }

    async fn exchange(
        &self,
        code: &str,
        pending: &OidcPending,
    ) -> Result<VerifiedGoogleIdentity, AuthError> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .map_err(|_| AuthError::InvalidIdentityProof)?
            .set_pkce_verifier(PkceCodeVerifier::new(pending.pkce_verifier.clone()))
            .request_async(&self.http_client)
            .await
            .map_err(|_| AuthError::InvalidIdentityProof)?;
        let id_token = token_response
            .extra_fields()
            .id_token()
            .ok_or(AuthError::InvalidIdentityProof)?;
        let claims = id_token
            .claims(
                &self.client.id_token_verifier(),
                &Nonce::new(pending.nonce.clone()),
            )
            .map_err(|_| AuthError::InvalidIdentityProof)?;
        let email = claims
            .email()
            .map(|email| email.as_str().to_owned())
            .ok_or(AuthError::InvalidIdentityProof)?;
        VerifiedGoogleIdentity::from_verified_claims(
            claims.issuer().to_string(),
            claims.subject().to_string(),
            email,
            claims.email_verified().unwrap_or(false),
        )
    }
}

#[derive(Debug)]
pub struct SiweWalletProofVerifier;

impl WalletProofVerifier for SiweWalletProofVerifier {
    fn verify(&self, message: &str, signature: &str) -> Result<String, AuthError> {
        let message = Message::from_str(message).map_err(|_| AuthError::InvalidWalletProof)?;
        if !message.valid_now() {
            return Err(AuthError::ChallengeExpired);
        }
        let signature = signature.strip_prefix("0x").unwrap_or(signature);
        let signature = hex::decode(signature).map_err(|_| AuthError::InvalidWalletProof)?;
        let signature: [u8; 65] = signature
            .try_into()
            .map_err(|_| AuthError::InvalidWalletProof)?;
        message
            .verify_eip191(&signature)
            .map_err(|_| AuthError::InvalidWalletProof)?;
        Ok(format!("0x{}", hex::encode(message.address)))
    }
}
