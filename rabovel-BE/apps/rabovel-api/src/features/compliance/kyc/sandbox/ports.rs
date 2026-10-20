use std::sync::Arc;
#[cfg(debug_assertions)]
pub(crate) trait SandboxKyc {
    fn sandbox_kyc(&self) -> &Option<Arc<kyc::MockKycProvider>>;
}
