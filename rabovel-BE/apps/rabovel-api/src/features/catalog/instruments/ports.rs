use std::sync::Arc;
pub(crate) trait CatalogSource {
    fn instrument_provider(
        &self,
    ) -> &Arc<dyn crate::features::catalog::instruments::ports::InstrumentProvider>;
}

use super::models::Instrument;

use super::models::InstrumentCatalog;

use async_trait::async_trait;

#[async_trait]
pub trait InstrumentProvider: Send + Sync {
    async fn catalog(&self) -> Result<InstrumentCatalog, String>;

    /// One instrument by ticker, case-insensitive.
    async fn find(&self, ticker: &str) -> Result<Option<Instrument>, String> {
        Ok(self
            .catalog()
            .await?
            .instruments
            .into_iter()
            .find(|instrument| instrument.ticker.eq_ignore_ascii_case(ticker)))
    }
}
