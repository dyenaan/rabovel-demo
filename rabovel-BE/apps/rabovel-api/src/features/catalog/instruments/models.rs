use serde::Serialize;
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
