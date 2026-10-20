use crate::app::errors::ApplicationError;
use crate::app::support::unix_now;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::market_data::charts::calculations as market_views;
use crate::features::market_data::charts::usecases::RangeQuery;
use crate::features::market_data::markets::usecases::trade_points;
use crate::features::market_data::prices::usecases::price_path;
use crate::features::portfolio::performance::dto::PerformancePointResponse;
use crate::features::portfolio::valuation::usecases::portfolio_view;

pub(crate) async fn portfolio_performance(
    state: &impl Ports,
    user: AuthenticatedUser,
    query: RangeQuery,
) -> Result<Vec<PerformancePointResponse>, ApplicationError> {
    let view = portfolio_view(state, &user.user_id).await?;
    let (bucket, count) = market_views::range_buckets(query.range.as_deref().unwrap_or("1M"));
    let now = unix_now();
    let mut values = vec![view.cash; count];
    for position in &view.positions {
        let trades = trade_points(state, &position.ticker).await?;
        let series = market_views::candles(
            &trades,
            price_path(state, &position.ticker),
            now,
            bucket,
            count,
        );
        for (value, candle) in values.iter_mut().zip(series) {
            *value += position.quantity * i128::from(candle.close);
        }
    }
    let start = (now / bucket + 1) * bucket - bucket * count as u64;
    Ok(values
        .into_iter()
        .enumerate()
        .map(|(index, value)| PerformancePointResponse {
            date: market_views::iso_date(start + index as u64 * bucket),
            portfolio_value: market_views::fixed2(value, state.price_decimals()),
        })
        .collect())
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::assets::listings::ports::PublishedAssets
    + crate::features::portfolio::balances::ports::Balances
    + crate::features::trading::ports::HouseSpread
    + crate::features::catalog::instruments::ports::CatalogSource
    + crate::features::issuer::assets::issuance::inventory::ports::Inventory
    + crate::features::market_data::markets::ports::ExecutionMarket
    + crate::features::trading::orders::ports::Orders
    + crate::features::payments::cngn::ports::PaymentProvider
    + crate::features::issuer::assets::issuance::inventory::ports::Positions
    + crate::features::market_data::prices::ports::PriceDecimals
    + crate::features::market_data::prices::ports::ReferencePrices
    + crate::features::issuer::assets::issuance::setup::ports::SetupLookup
    + crate::features::identity::wallets::queries::ports::WalletLookup
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::assets::listings::ports::PublishedAssets
            + crate::features::portfolio::balances::ports::Balances
            + crate::features::trading::ports::HouseSpread
            + crate::features::catalog::instruments::ports::CatalogSource
            + crate::features::issuer::assets::issuance::inventory::ports::Inventory
            + crate::features::market_data::markets::ports::ExecutionMarket
            + crate::features::trading::orders::ports::Orders
            + crate::features::payments::cngn::ports::PaymentProvider
            + crate::features::issuer::assets::issuance::inventory::ports::Positions
            + crate::features::market_data::prices::ports::PriceDecimals
            + crate::features::market_data::prices::ports::ReferencePrices
            + crate::features::issuer::assets::issuance::setup::ports::SetupLookup
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
