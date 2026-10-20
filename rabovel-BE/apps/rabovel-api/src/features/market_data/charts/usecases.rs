use crate::app::errors::ApplicationError;
use crate::app::support::unix_now;
use crate::features::catalog::instruments::usecases::instrument_by_asset;
use crate::features::catalog::instruments::usecases::instrument_by_market;
use crate::features::market_data::charts::calculations as market_views;
use crate::features::market_data::charts::dto::NavPointResponse;
use crate::features::market_data::charts::dto::PricePointResponse;
use crate::features::market_data::markets::usecases::trade_points;
use crate::features::market_data::prices::usecases::price_path;
use crate::features::market_data::prices::usecases::reference_price;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct RangeQuery {
    pub(crate) range: Option<String>,
}

pub(crate) async fn market_price_history(
    state: &impl Ports,
    market_id: String,
    query: RangeQuery,
) -> Result<Vec<PricePointResponse>, ApplicationError> {
    let instrument = instrument_by_market(state, &market_id).await?;
    let (bucket, count) = market_views::range_buckets(query.range.as_deref().unwrap_or("1D"));
    reference_price(state, &instrument.ticker)?;
    let trades = trade_points(state, &instrument.ticker).await?;
    let decimals = state.price_decimals();
    let price = |units: u64| market_views::fixed2(units.into(), decimals);
    Ok(market_views::candles(
        &trades,
        price_path(state, &instrument.ticker),
        unix_now(),
        bucket,
        count,
    )
    .into_iter()
    .map(|candle| PricePointResponse {
        timestamp: market_views::iso(candle.start),
        open: price(candle.open),
        high: price(candle.high),
        low: price(candle.low),
        close: price(candle.close),
        volume: candle.volume.to_string(),
    })
    .collect())
}

pub(crate) async fn asset_nav_history(
    state: &impl Ports,
    asset_id: String,
) -> Result<Vec<NavPointResponse>, ApplicationError> {
    let instrument = instrument_by_asset(state, &asset_id).await?;
    let (bucket, count) = market_views::range_buckets("3M");
    reference_price(state, &instrument.ticker)?;
    let trades = trade_points(state, &instrument.ticker).await?;
    Ok(market_views::candles(
        &trades,
        price_path(state, &instrument.ticker),
        unix_now(),
        bucket,
        count,
    )
    .into_iter()
    .map(|candle| NavPointResponse {
        date: market_views::iso_date(candle.start),
        nav: market_views::fixed2(candle.close.into(), state.price_decimals()),
    })
    .collect())
}
pub(crate) trait Ports:
    crate::features::trading::ports::HouseSpread
    + crate::features::catalog::instruments::ports::CatalogSource
    + crate::features::market_data::markets::ports::ExecutionMarket
    + crate::features::market_data::prices::ports::PriceDecimals
    + crate::features::market_data::prices::ports::ReferencePrices
{
}
impl<
        T: crate::features::trading::ports::HouseSpread
            + crate::features::catalog::instruments::ports::CatalogSource
            + crate::features::market_data::markets::ports::ExecutionMarket
            + crate::features::market_data::prices::ports::PriceDecimals
            + crate::features::market_data::prices::ports::ReferencePrices
            + ?Sized,
    > Ports for T
{
}
