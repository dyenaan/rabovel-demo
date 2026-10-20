use crate::features::catalog::instruments::fixtures::ngx::*;
use crate::features::catalog::instruments::models::*;
use crate::features::catalog::instruments::ports::InstrumentProvider;
use crate::features::catalog::instruments::providers::ngx_simulated::SimulatedInstrumentProvider;
use crate::features::catalog::instruments::resolvers::*;
use async_trait::async_trait;
use issuer_assets::assets::drafts::models::is_valid_ticker;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use std::collections::HashSet;
use std::sync::Arc;

fn issuer_instrument(asset: AssetDraftRecord) -> Instrument {
    Instrument {
        market_id: market_id(&asset.draft.market, &asset.draft.ticker),
        instrument_code: asset.draft.instrument_code,
        ticker: asset.draft.ticker,
        name: asset.draft.name,
        market: asset.draft.market,
        share_class: asset.draft.share_class,
        asset_type: asset.draft.asset_type,
        settlement_currency: asset.draft.settlement_currency,
        origin: InstrumentOrigin::Issuer,
        asset_id: asset.asset_id,
        mint_address: asset.mint_address,
    }
}
/// One catalog for every tradable instrument: the NGX-listed instruments plus
/// every issuer asset that is minted and published live. A newly listed issuer
/// asset becomes tradable without any registration step.
pub struct UnifiedInstrumentCatalog {
    repository: Arc<dyn issuer_assets::assets::drafts::ports::AssetQueries>,
}
impl UnifiedInstrumentCatalog {
    pub fn new(repository: Arc<dyn issuer_assets::assets::drafts::ports::AssetQueries>) -> Self {
        Self { repository }
    }

    async fn live_issuer_instruments(&self) -> Result<Vec<Instrument>, String> {
        let assets = self
            .repository
            .list_minted_assets()
            .await
            .map_err(|error| format!("could not load listed assets: {error:?}"))?;
        // Listing rejects ticker collisions; this filter also keeps records
        // from before the ticker rule (e.g. `RABO-NG-TELCO`) out of matching.
        let mut seen = HashSet::new();
        Ok(assets
            .into_iter()
            .filter(|asset| asset.draft.listing_status == "live")
            .filter(|asset| is_valid_ticker(&asset.draft.ticker))
            .filter(|asset| !is_ngx_listed(&asset.draft.ticker))
            .filter(|asset| seen.insert(asset.draft.ticker.clone()))
            .map(issuer_instrument)
            .collect())
    }
}
#[async_trait]
impl InstrumentProvider for UnifiedInstrumentCatalog {
    async fn catalog(&self) -> Result<InstrumentCatalog, String> {
        let mut instruments = SimulatedInstrumentProvider.catalog().await?.instruments;
        instruments.extend(self.live_issuer_instruments().await?);
        Ok(InstrumentCatalog {
            provider: "rabovel_unified_instrument_catalog".into(),
            simulated: true,
            as_of: now(),
            instruments,
        })
    }

    async fn find(&self, ticker: &str) -> Result<Option<Instrument>, String> {
        if let Some(listed) = NGX_LISTED_TICKERS
            .iter()
            .find(|listed| listed.eq_ignore_ascii_case(ticker))
        {
            return Ok(Some(ngx_instrument(listed)));
        }
        Ok(self
            .live_issuer_instruments()
            .await?
            .into_iter()
            .find(|instrument| instrument.ticker.eq_ignore_ascii_case(ticker)))
    }
}
