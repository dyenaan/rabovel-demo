use crate::*;
use async_trait::async_trait;

#[async_trait]
pub trait KycProviderApi: Send + Sync {
    async fn submit_case(&self, submission: KycSubmission) -> Result<KycCaseHandle, KycError>;

    /// Verifies an inbound webhook's authenticity over the *raw* body bytes
    /// before any structured parsing occurs, then returns a verdict.
    /// Callers must pass the untouched request body -- never a
    /// re-serialized/re-parsed form of it -- since the signature covers the
    /// exact bytes the vendor sent.
    fn verify_webhook(
        &self,
        raw_body: &[u8],
        signature_header: &str,
    ) -> Result<VerifiedKycVerdict, KycError>;
}
