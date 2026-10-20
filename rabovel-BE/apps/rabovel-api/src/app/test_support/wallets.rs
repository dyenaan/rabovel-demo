use identity::wallets::linking::ports::WalletProofVerifier;
use identity::AuthError;
use siwe::Message;
use std::str::FromStr;

pub(crate) struct FakeWalletProof;

impl WalletProofVerifier for FakeWalletProof {
    fn verify(&self, message: &str, signature: &str) -> Result<String, AuthError> {
        if signature != "valid-test-signature" {
            return Err(AuthError::InvalidWalletProof);
        }
        let message = Message::from_str(message).map_err(|_| AuthError::InvalidWalletProof)?;
        Ok(format!("0x{}", hex::encode(message.address)))
    }
}
