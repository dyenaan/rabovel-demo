use crate::app::errors::ApplicationError;
use crate::features::catalog::instruments::usecases::catalog_unavailable;

pub(crate) const BPS_DENOMINATOR: u128 = 10_000;

/// Display price in whole payment units, two decimals, rounded half up.
pub(crate) fn display_price(base_units: u128, decimals: u8) -> String {
    let scale = 10_u128.pow(u32::from(decimals));
    let cents = (base_units * 100 + scale / 2) / scale;
    format!("{}.{:02}", cents / 100, cents % 100)
}

pub(crate) fn quote_asset(settlement_currency: &str) -> &str {
    match settlement_currency {
        "CNGN" => "NGN",
        "USDC" => "USD",
        "CADC" => "CAD",
        other => other,
    }
}

/// Current reference price for a ticker, in payment-asset base units.
pub(crate) fn reference_price(state: &impl Ports, ticker: &str) -> Result<u64, ApplicationError> {
    state
        .price_feed()
        .latest(ticker, state.price_decimals())
        .map(|price| price.price_base_units)
        .map_err(catalog_unavailable)
}

/// The ticker's simulated price at any unix second. Callers validate the
/// ticker with [`reference_price`] first; a failed sample reads as 0.
pub(crate) fn price_path<'a>(state: &'a impl Ports, ticker: &'a str) -> impl Fn(u64) -> u64 + 'a {
    move |at| {
        state
            .price_feed()
            .price_at(ticker, state.price_decimals(), at)
            .map_or(0, |price| price.price_base_units)
    }
}
pub(crate) trait Ports:
    crate::features::market_data::prices::ports::PriceDecimals
    + crate::features::market_data::prices::ports::ReferencePrices
{
}
impl<
        T: crate::features::market_data::prices::ports::PriceDecimals
            + crate::features::market_data::prices::ports::ReferencePrices
            + ?Sized,
    > Ports for T
{
}
