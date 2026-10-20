//! Compliance verification contracts and owned provider/persistence implementations.

pub mod cases;
mod errors;
pub mod ports;
pub mod providers;
pub mod verdicts;
pub use cases::models::{DocumentType, KycCaseHandle, KycSubmission};
pub use errors::KycError;
pub use ports::KycProviderApi;
pub use providers::mock::MockKycProvider;
pub use verdicts::models::VerifiedKycVerdict;
#[cfg(test)]
mod tests;
