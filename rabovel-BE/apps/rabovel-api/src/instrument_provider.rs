use async_trait::async_trait;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Instrument {
    pub instrument_code: String,
    pub ticker: String,
    pub name: String,
    pub market: String,
    pub share_class: String,
    pub asset_type: String,
    pub settlement_currency: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct InstrumentCatalog {
    pub provider: String,
    pub simulated: bool,
    pub as_of: u64,
    pub instruments: Vec<Instrument>,
}

#[async_trait]
pub trait InstrumentProvider: Send + Sync {
    async fn catalog(&self) -> Result<InstrumentCatalog, String>;
}

/// Deterministic instrument fixtures; no NGX connection or issuer action is required.
pub struct SimulatedInstrumentProvider;

#[async_trait]
impl InstrumentProvider for SimulatedInstrumentProvider {
    async fn catalog(&self) -> Result<InstrumentCatalog, String> {
        Ok(InstrumentCatalog {
            provider: "rabovel_simulated_instrument_provider".into(),
            simulated: true,
            as_of: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            instruments: vec![
                instrument("NGX-DANGCEM", "DANGCEM", "Dangote Cement Plc"),
                instrument("NGX-GTCO", "GTCO", "Guaranty Trust Holding Company Plc"),
                instrument("NGX-ZENITHBANK", "ZENITHBANK", "Zenith Bank Plc"),
                instrument("NGX-MTNN", "MTNN", "MTN Nigeria Communications Plc"),
            ],
        })
    }
}

fn instrument(instrument_code: &str, ticker: &str, name: &str) -> Instrument {
    Instrument {
        instrument_code: instrument_code.into(),
        ticker: ticker.into(),
        name: name.into(),
        market: "NGX".into(),
        share_class: "Ordinary Shares".into(),
        asset_type: "equity".into(),
        settlement_currency: "NGN".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn simulated_catalog_is_explicitly_marked_and_repeatable() {
        let provider = SimulatedInstrumentProvider;
        let first = provider.catalog().await.unwrap();
        let second = provider.catalog().await.unwrap();

        assert_eq!(first.provider, "rabovel_simulated_instrument_provider");
        assert!(first.simulated);
        assert_eq!(first.instruments, second.instruments);
        assert_eq!(first.instruments.len(), 4);
        assert!(first.instruments.iter().all(|instrument| {
            instrument.market == "NGX" && instrument.settlement_currency == "NGN"
        }));
    }
}
