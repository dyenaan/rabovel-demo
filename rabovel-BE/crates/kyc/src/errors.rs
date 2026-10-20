#[derive(Debug, thiserror::Error)]
pub enum KycError {
    #[error("the KYC case was not found")]
    CaseNotFound,
    #[error("the webhook signature is invalid")]
    InvalidSignature,
    #[error("the webhook payload could not be parsed")]
    MalformedPayload,
    #[error("the KYC provider is unavailable: {0}")]
    Unavailable(String),
}
