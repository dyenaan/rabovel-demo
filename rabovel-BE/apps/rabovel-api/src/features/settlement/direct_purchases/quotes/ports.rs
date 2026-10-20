pub(crate) trait QuotePolicy {
    fn quote_config(
        &self,
    ) -> settlement_orchestrator::direct_purchases::quotes::models::QuoteConfig;
}
