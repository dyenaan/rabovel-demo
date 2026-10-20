use crate::features::catalog::instruments::fixtures::ngx::*;
use crate::features::catalog::instruments::models::*;
use crate::features::catalog::instruments::ports::InstrumentProvider;
use crate::features::catalog::instruments::resolvers::*;
use async_trait::async_trait;

/// The NGX-listed instruments only.
pub struct SimulatedInstrumentProvider;
#[async_trait]
impl InstrumentProvider for SimulatedInstrumentProvider {
    async fn catalog(&self) -> Result<InstrumentCatalog, String> {
        Ok(InstrumentCatalog {
            provider: "rabovel_simulated_instrument_provider".into(),
            simulated: true,
            as_of: now(),
            instruments: NGX_LISTED_TICKERS
                .iter()
                .map(|t| ngx_instrument(t))
                .collect(),
        })
    }
}
