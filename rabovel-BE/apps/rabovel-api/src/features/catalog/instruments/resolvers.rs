use crate::features::catalog::instruments::fixtures::ngx::*;
use crate::features::catalog::instruments::models::*;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

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
pub(crate) fn ngx_instrument(ticker: &str) -> Instrument {
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
pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
