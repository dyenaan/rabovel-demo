use thiserror::Error;

#[derive(Debug, Error)]
pub enum IssuanceError {
    #[error("invalid chain configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Solana RPC failure: {0}")]
    Rpc(String),
    #[error("Solana instruction failure: {0}")]
    Instruction(String),
    #[error("Solana transaction failure: {0}")]
    Transaction(String),
}
