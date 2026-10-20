use crate::wallets::linking::ports::WalletProofVerifier;
use crate::AuthError;
use siwe::Message;
use std::str::FromStr;

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
