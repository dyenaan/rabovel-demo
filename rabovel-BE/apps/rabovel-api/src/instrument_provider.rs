use async_trait::async_trait;
use domain::assets::is_valid_ticker;
use identity::{AssetDraftRecord, Repository};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

/// Where a tradable instrument comes from. Both kinds are one asset class:
/// same ticker format, same catalog, same matching engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentOrigin {
    /// NGX-listed equity, modeled as already minted and tradable.
    NgxListed,
    /// Asset an issuer tokenized and published live through the issuer flow.
    Issuer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Instrument {
    /// Stable market identifier shared with the frontend watchlist,
    /// e.g. `mkt_ngx_dangcem`.
    pub market_id: String,
    pub instrument_code: String,
    pub ticker: String,
    pub name: String,
    pub market: String,
    pub share_class: String,
    pub asset_type: String,
    pub settlement_currency: String,
    pub origin: InstrumentOrigin,
    /// Asset id used by `/assets/:id`: the issuer asset id, or
    /// `ast_ngx_<ticker>` for NGX-listed instruments.
    pub asset_id: String,
    /// On-chain mint; NGX-listed instruments are assumed minted but have no
    /// Devnet mint yet.
    pub mint_address: Option<String>,
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

/// NGX-listed equities shown on the frontend watchlist. Deterministic
/// fixtures; no NGX connection or issuer action is required.
pub const NGX_LISTED_TICKERS: &[&str] = &[
    "ABBEYBANK", "ABCTRANS", "ACADEMY", "ACCESSCORP", "AFRINSURE", "AFRIPRUD", "AFROMEDIA",
    "AIICO", "AIRTELAFRI", "ALEX", "ARADEL", "AUSTINLAZ", "AVACAP", "AVAIF", "BAPLC",
    "BERGER", "BETAGLAS", "BUACEMENT", "BUAFOODS", "CADBURY", "CAP", "CAVERTON", "CHAMPION",
    "CHAMS", "CHELLARAM", "CILEASING", "CMFC", "CNIF", "CONHALLPLC", "CONOIL", "CORNERST",
    "CUSTODIAN", "CUTIX", "CWG", "DAARCOMM", "DANGCEM", "DANGSUGAR", "EKOCORP", "ELLAHLAKES",
    "ENAMELWA", "ETERNA", "ETI", "ETRANZACT", "EUNISELL", "FCMB", "FIDELITYBK", "FIDSON",
    "FIRSTHOLDCO", "FTGINSURE", "FTNCOCOA", "GEREGU", "GOLDBREW", "GTCO", "GUINEAINS",
    "GUINNESS", "HBMNG", "HMCALL", "HONYFLOUR", "IKEJAHOTEL", "IMG", "INFINITY", "INTBREW",
    "INTENEGINS", "JAIZBANK", "JAPAULGOLD", "JBERGER", "JOHNHOLT", "JULI", "LASACO", "LEARNAFRCA",
    "LEGENDINT", "LINKASSURE", "LIVESTOCK", "LIVINGTRUST", "MANSARD", "MAYBAKER", "MBENEFIT",
    "MCNICHOLS", "MECURE", "MEYER", "MOFIREIF", "MORISON", "MTNN", "MULTITREX", "MULTIVERSE",
    "NAHCO", "NASCON", "NB", "NCR", "NEIMETH", "NEM", "NESTLE", "NGXGROUP", "NIDF", "NNFM",
    "NPFMCRFBK", "NREIT", "NSLTECH", "OANDO", "OKOMUOIL", "OMATEK", "PHARMDEKO", "PREMPAINTS",
    "PRESCO", "PRESTIGE", "PZ", "REDSTAREX", "REGALINS", "RONCHESS", "ROYALEX", "RTBRISCOE",
    "SCOA", "SEPLAT", "SFSREIT", "SKYAVN", "SOVRENINS", "STANBIC", "STERLINGNG", "SUNUASSUR",
    "TANTALIZER", "THOMASWY", "TIP", "TOTAL", "TRANSCOHOT", "TRANSCORP", "TRANSEXPR", "TRANSPOWER",
    "TRIPPLEG", "UACN", "UBA", "UCAP", "UHOMREIT", "UNILEVER", "UNIONDICON", "UNITYBNK",
    "UNIVINSURE", "UPDC", "UPDCREIT", "UPL", "VERITASKAP", "VFDGROUP", "VITAFOAM", "WAPIC",
    "WEMABANK", "ZENITHBANK", "ZICHIS",
];

const NGX_NAMES: &[(&str, &str)] = &[
    ("DANGCEM", "Dangote Cement Plc"),
    ("GTCO", "Guaranty Trust Holding Company Plc"),
    ("ZENITHBANK", "Zenith Bank Plc"),
    ("MTNN", "MTN Nigeria Communications Plc"),
];

pub fn is_ngx_listed(ticker: &str) -> bool {
    NGX_LISTED_TICKERS
        .iter()
        .any(|listed| listed.eq_ignore_ascii_case(ticker))
}

/// Market id shared by every catalog instrument, matching the frontend
/// watchlist format (`mkt_ngx_dangcem`).
pub fn market_id(market: &str, ticker: &str) -> String {
    format!(
        "mkt_{}_{}",
        market.to_ascii_lowercase(),
        ticker.to_ascii_lowercase()
    )
}

pub fn ngx_asset_id(ticker: &str) -> String {
    format!("ast_ngx_{}", ticker.to_ascii_lowercase())
}

fn ngx_instrument(ticker: &str) -> Instrument {
    let name = NGX_NAMES
        .iter()
        .find(|(listed, _)| *listed == ticker)
        .map_or("NGX-listed equity · simulated", |(_, name)| name);
    Instrument {
        market_id: market_id("NGX", ticker),
        instrument_code: format!("NGX-{ticker}"),
        ticker: ticker.into(),
        name: name.into(),
        market: "NGX".into(),
        share_class: "Ordinary Shares".into(),
        asset_type: "equity".into(),
        settlement_currency: "CNGN".into(),
        origin: InstrumentOrigin::NgxListed,
        asset_id: ngx_asset_id(ticker),
        mint_address: None,
    }
}

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

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// The NGX-listed instruments only.
pub struct SimulatedInstrumentProvider;

#[async_trait]
impl InstrumentProvider for SimulatedInstrumentProvider {
    async fn catalog(&self) -> Result<InstrumentCatalog, String> {
        Ok(InstrumentCatalog {
            provider: "rabovel_simulated_instrument_provider".into(),
            simulated: true,
            as_of: now(),
            instruments: NGX_LISTED_TICKERS.iter().map(|t| ngx_instrument(t)).collect(),
        })
    }
}

/// One catalog for every tradable instrument: the NGX-listed instruments plus
/// every issuer asset that is minted and published live. A newly listed issuer
/// asset becomes tradable without any registration step.
pub struct UnifiedInstrumentCatalog {
    repository: Arc<dyn Repository>,
}

impl UnifiedInstrumentCatalog {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use domain::assets::{AssetMetadata, NewAssetDraft, DEMO_DISCLOSURE};
    use identity::InMemoryRepository;

    fn issuer_asset(asset_id: &str, ticker: &str, status: &str, listing: &str) -> AssetDraftRecord {
        AssetDraftRecord {
            asset_id: asset_id.into(),
            issuer_user_id: "usr_issuer".into(),
            canonical_key: format!("NGX:{ticker}:ORDINARY"),
            draft: NewAssetDraft {
                instrument_code: format!("{ticker}-CODE"),
                name: format!("{ticker} Demo"),
                ticker: ticker.into(),
                market: "NGX".into(),
                share_class: "Ordinary".into(),
                asset_type: "equity".into(),
                decimals: 0,
                authorized_units: 1_000,
                settlement_currency: "CNGN".into(),
                representation: "1 token = 1 simulated share".into(),
                rights_description: "Simulated".into(),
                disclosure: DEMO_DISCLOSURE.into(),
                metadata: AssetMetadata {
                    name: format!("{ticker} Demo"),
                    symbol: ticker.into(),
                    description: "Demo".into(),
                    image_uri: None,
                    external_url: None,
                    metadata_uri: None,
                    additional_metadata: vec![],
                },
                backing: None,
                listing_status: listing.into(),
            },
            status: status.into(),
            created_at: 1,
            updated_at: 1,
            mint_address: Some(format!("mint-{asset_id}")),
        }
    }

    #[tokio::test]
    async fn ngx_catalog_matches_the_watchlist_format() {
        let catalog = SimulatedInstrumentProvider.catalog().await.unwrap();
        assert_eq!(catalog.provider, "rabovel_simulated_instrument_provider");
        assert!(catalog.simulated);
        assert_eq!(catalog.instruments.len(), 146);
        assert!(catalog.instruments.iter().all(|instrument| {
            is_valid_ticker(&instrument.ticker)
                && instrument.market_id == format!("mkt_ngx_{}", instrument.ticker.to_lowercase())
                && instrument.origin == InstrumentOrigin::NgxListed
                && instrument.asset_id == format!("ast_ngx_{}", instrument.ticker.to_lowercase())
                && instrument.settlement_currency == "CNGN"
        }));
    }

    #[tokio::test]
    async fn live_issuer_assets_join_the_catalog_automatically() {
        let repository = Arc::new(InMemoryRepository::default());
        for asset in [
            issuer_asset("ast_live", "RABOTELCO", "minted", "live"),
            issuer_asset("ast_unlisted", "RABOBANK", "minted", "not_listed"),
            issuer_asset("ast_legacy", "RABO-NG-TELCO", "minted", "live"),
        ] {
            repository.create_asset_draft(asset).await.unwrap();
        }
        let catalog = UnifiedInstrumentCatalog::new(repository);

        let instruments = catalog.catalog().await.unwrap().instruments;
        assert_eq!(instruments.len(), 147);
        let live = catalog.find("rabotelco").await.unwrap().unwrap();
        assert_eq!(live.origin, InstrumentOrigin::Issuer);
        assert_eq!(live.market_id, "mkt_ngx_rabotelco");
        assert_eq!(live.asset_id, "ast_live");
        assert_eq!(live.mint_address.as_deref(), Some("mint-ast_live"));
        assert!(catalog.find("RABOBANK").await.unwrap().is_none());
        assert!(catalog.find("RABO-NG-TELCO").await.unwrap().is_none());
        assert_eq!(
            catalog.find("gtco").await.unwrap().unwrap().origin,
            InstrumentOrigin::NgxListed
        );
    }
}
