use std::sync::Arc;
pub(crate) trait CaseStore {
    fn cases(&self) -> &Arc<dyn kyc::cases::ports::CaseRepository>;
}
pub(crate) trait KycProvider {
    fn kyc_provider(&self) -> &Arc<dyn kyc::KycProviderApi>;
}
