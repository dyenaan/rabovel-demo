use crate::app::constants::WALLET_CHALLENGE_TTL_SECONDS;
use crate::app::errors::ApplicationError;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::wallets::linking::dto::SolanaWalletChallengeRequest;
use crate::features::identity::wallets::linking::dto::WalletChallengeRequest;
use crate::features::identity::wallets::linking::dto::WalletChallengeResponse;
use crate::features::identity::wallets::linking::dto::WalletLinkRequest;
use crate::features::identity::wallets::linking::dto::WalletLinkResponse;
use base64::engine::general_purpose::STANDARD as BASE64;
use ed25519_dalek::Signature as Ed25519Signature;
use ed25519_dalek::VerifyingKey;
use events::schema::wallet::WalletLinked;
use identity::wallets::linking::models::WalletChallengeRecord;
use identity::AuthError;
use identity::SupportedChain;
use identity::WalletConnection;
use identity::WalletProvider;
use siwe::Message;
use time::format_description::well_known::Rfc3339;
use time::Duration;
use time::OffsetDateTime;

use crate::app::support::random_hex;
use crate::app::support::serde_str;
use crate::app::support::state_unavailable;
use crate::app::support::unix_now;
use crate::features::compliance::onboarding::usecases::publish_onboarding_change;
use base64::Engine;
use ed25519_dalek::Verifier;
use std::str::FromStr;

pub(crate) async fn wallet_challenge(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: WalletChallengeRequest,
) -> Result<WalletChallengeResponse, ApplicationError> {
    if matches!(request.provider, WalletProvider::Phantom) {
        return Err(ApplicationError::from(AuthError::InvalidProvider));
    }
    let chain = SupportedChain::from_eip155_chain_id(request.chain_id)
        .ok_or_else(|| ApplicationError::from(AuthError::InvalidProvider))?;
    let address = request.address.to_ascii_lowercase();
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(ApplicationError::from(AuthError::InvalidWalletProof));
    }
    let address_bytes = hex::decode(&address[2..])
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    let address_bytes: [u8; 20] = address_bytes
        .try_into()
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    let checksum_address = siwe::eip55(&address_bytes);
    let challenge_id = random_hex(16).map_err(ApplicationError::from)?;
    let nonce = siwe::generate_nonce();
    let issued_at = OffsetDateTime::now_utc();
    let expiration_time = issued_at + Duration::seconds(WALLET_CHALLENGE_TTL_SECONDS as i64);
    let issued_at = issued_at
        .format(&Rfc3339)
        .map_err(|_| ApplicationError::from(state_unavailable()))?;
    let expiration_time = expiration_time
        .format(&Rfc3339)
        .map_err(|_| ApplicationError::from(state_unavailable()))?;
    let message = format!(
        "{} wants you to sign in with your Ethereum account:\n{}\n\nLink this wallet to your Rabovel brokerage account.\n\nURI: {}/wallet/link\nVersion: 1\nChain ID: {}\nNonce: {}\nIssued At: {}\nExpiration Time: {}",
        state.siwe_domain(),
        checksum_address,
        state.public_origin(),
        chain.eip155_chain_id().expect("EIP-155 chain"),
        nonce,
        issued_at,
        expiration_time,
    );
    // Validate server configuration before exposing the message to a wallet.
    Message::from_str(&message)
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    let expires_at = unix_now().saturating_add(WALLET_CHALLENGE_TTL_SECONDS);
    state
        .challenges()
        .put_wallet_challenge(
            &challenge_id,
            WalletChallengeRecord {
                user_id: user.user_id,
                expected_address: address,
                chain,
                provider: request.provider,
                message: message.clone(),
            },
            WALLET_CHALLENGE_TTL_SECONDS,
        )
        .await
        .map_err(ApplicationError::from)?;
    Ok(WalletChallengeResponse {
        challenge_id,
        message,
        expires_at,
    })
}

pub(crate) async fn solana_wallet_challenge(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: SolanaWalletChallengeRequest,
) -> Result<WalletChallengeResponse, ApplicationError> {
    let public_key = bs58::decode(&request.address)
        .into_vec()
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    VerifyingKey::from_bytes(&public_key)
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;

    let challenge_id = random_hex(16).map_err(ApplicationError::from)?;
    let nonce = random_hex(16).map_err(ApplicationError::from)?;
    let issued_at = OffsetDateTime::now_utc();
    let expiration_time = issued_at + Duration::seconds(WALLET_CHALLENGE_TTL_SECONDS as i64);
    let issued_at = issued_at
        .format(&Rfc3339)
        .map_err(|_| ApplicationError::from(state_unavailable()))?;
    let expiration_time = expiration_time
        .format(&Rfc3339)
        .map_err(|_| ApplicationError::from(state_unavailable()))?;
    let message = format!(
        "{} wants you to link this Solana wallet:\n{}\n\nLink this wallet to your Rabovel brokerage account.\n\nURI: {}/wallet/link\nNonce: {}\nIssued At: {}\nExpiration Time: {}",
        state.siwe_domain(),
        request.address,
        state.public_origin(),
        nonce,
        issued_at,
        expiration_time,
    );
    let expires_at = unix_now().saturating_add(WALLET_CHALLENGE_TTL_SECONDS);
    state
        .challenges()
        .put_wallet_challenge(
            &challenge_id,
            WalletChallengeRecord {
                user_id: user.user_id,
                expected_address: request.address,
                chain: SupportedChain::Solana,
                provider: WalletProvider::Phantom,
                message: message.clone(),
            },
            WALLET_CHALLENGE_TTL_SECONDS,
        )
        .await
        .map_err(ApplicationError::from)?;
    Ok(WalletChallengeResponse {
        challenge_id,
        message,
        expires_at,
    })
}

pub(crate) async fn wallet_link(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: WalletLinkRequest,
) -> Result<WalletLinkResponse, ApplicationError> {
    if state
        .challenges()
        .is_wallet_challenge_used(&request.challenge_id)
        .await
        .map_err(ApplicationError::from)?
    {
        return Err(ApplicationError::from(AuthError::ReplayDetected));
    }
    let challenge = state
        .challenges()
        .take_wallet_challenge(&request.challenge_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::InvalidWalletProof))?;
    state
        .challenges()
        .mark_wallet_challenge_used(&request.challenge_id, WALLET_CHALLENGE_TTL_SECONDS)
        .await
        .map_err(ApplicationError::from)?;
    if challenge.user_id != user.user_id {
        return Err(ApplicationError::from(AuthError::WalletNotOwned));
    }
    let signer = if matches!(challenge.chain, SupportedChain::Solana) {
        verify_solana_signature(
            &challenge.expected_address,
            challenge.message.as_bytes(),
            &request.signature,
        )?;
        challenge.expected_address.clone()
    } else {
        state
            .wallet_proof()
            .verify(&challenge.message, &request.signature)
            .map_err(ApplicationError::from)?
    };
    if !signer.eq_ignore_ascii_case(&challenge.expected_address) {
        return Err(ApplicationError::from(AuthError::InvalidWalletProof));
    }
    let wallet_id = format!("wal_{}", random_hex(16).map_err(ApplicationError::from)?);
    let wallet = WalletConnection::from_verified_proof(
        wallet_id.clone(),
        user.user_id.clone(),
        challenge.chain,
        signer,
        challenge.provider,
        unix_now(),
    )
    .map_err(ApplicationError::from)?;
    state
        .wallets()
        .insert_wallet(wallet.clone())
        .await
        .map_err(ApplicationError::from)?;
    let mut record = state
        .accounts()
        .get_user(&user.user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    let before_status = state.auth_service().onboarding_status(&record.profile);
    record.profile.record_verified_wallet(wallet_id);
    state
        .profile_updates()
        .save_profile(record.profile.clone())
        .await
        .map_err(ApplicationError::from)?;
    let after_status = state.auth_service().onboarding_status(&record.profile);

    let _ = events::publish(
        state.event_producer().as_ref(),
        WalletLinked {
            user_id: user.user_id.clone(),
            wallet_id: wallet.wallet_id().to_string(),
            chain: serde_str(&wallet.chain()),
            provider: serde_str(&wallet.provider()),
            verified_at: wallet.verified_at(),
        },
        "order-gateway",
    )
    .await;
    publish_onboarding_change(state, &user.user_id, before_status, after_status).await;

    Ok(WalletLinkResponse {
        onboarding_status: after_status,
        wallet,
    })
}

pub(crate) fn verify_solana_signature(
    address: &str,
    message: &[u8],
    signature: &str,
) -> Result<(), ApplicationError> {
    let public_key = bs58::decode(address)
        .into_vec()
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    let verifying_key = VerifyingKey::from_bytes(&public_key)
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    let signature = BASE64
        .decode(signature)
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    let signature = Ed25519Signature::from_slice(&signature)
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))?;
    verifying_key
        .verify(message, &signature)
        .map_err(|_| ApplicationError::from(AuthError::InvalidWalletProof))
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::identity::accounts::ports::AccountStore
    + crate::features::identity::access::ports::AccessPolicy
    + crate::features::identity::wallets::linking::ports::ChallengeStore
    + crate::features::compliance::onboarding::ports::EventPublisher
    + crate::features::identity::accounts::ports::ProfileUpdates
    + crate::features::identity::wallets::linking::ports::WalletOrigin
    + crate::features::identity::wallets::linking::ports::WalletDomain
    + crate::features::identity::wallets::linking::ports::WalletProof
    + crate::features::identity::wallets::queries::ports::WalletStore
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::identity::accounts::ports::AccountStore
            + crate::features::identity::access::ports::AccessPolicy
            + crate::features::identity::wallets::linking::ports::ChallengeStore
            + crate::features::compliance::onboarding::ports::EventPublisher
            + crate::features::identity::accounts::ports::ProfileUpdates
            + crate::features::identity::wallets::linking::ports::WalletOrigin
            + crate::features::identity::wallets::linking::ports::WalletDomain
            + crate::features::identity::wallets::linking::ports::WalletProof
            + crate::features::identity::wallets::queries::ports::WalletStore
            + ?Sized,
    > Ports for T
{
}
