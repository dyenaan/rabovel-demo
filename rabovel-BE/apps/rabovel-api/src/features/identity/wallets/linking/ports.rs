use std::sync::Arc;
pub(crate) trait WalletProof {
    fn wallet_proof(&self) -> &Arc<dyn identity::wallets::linking::ports::WalletProofVerifier>;
}
pub(crate) trait WalletDomain {
    fn siwe_domain(&self) -> &Arc<str>;
}
pub(crate) trait WalletOrigin {
    fn public_origin(&self) -> &Arc<str>;
}
pub(crate) trait ChallengeStore {
    fn challenges(&self) -> &Arc<dyn identity::wallets::linking::ports::ChallengeRepository>;
}
