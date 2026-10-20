use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::app::support::unix_now;
use crate::features::catalog::instruments::usecases::instrument_by_market;
use crate::features::catalog::instruments::Instrument;
use crate::features::market_data::charts::calculations as market_views;
use crate::features::market_data::markets::dto::MarketDetailResponse;
use crate::features::market_data::markets::dto::MarketListResponse;
use crate::features::market_data::markets::dto::MarketResponse;
use crate::features::market_data::markets::dto::OrderBookLevelResponse;
use crate::features::market_data::markets::dto::OrderBookResponse;
use crate::features::market_data::markets::dto::PublicTradeResponse;
use crate::features::market_data::prices::usecases::display_price;
use crate::features::market_data::prices::usecases::price_path;
use crate::features::market_data::prices::usecases::quote_asset;
use crate::features::market_data::prices::usecases::BPS_DENOMINATOR;
use identity::AuthError;
use portfolio_trading::OrderSide;

pub(crate) async fn market_response(
    state: &impl Ports,
    instrument: Instrument,
) -> Result<MarketResponse, ApplicationError> {
    let decimals = state.price_decimals();
    let price = state
        .price_feed()
        .latest(&instrument.ticker, decimals)
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })?;
    // 24h statistics: the price path sampled every 5 minutes plus engine trades.
    let day_ago = price.as_of.saturating_sub(86_400);
    let ticker = instrument.ticker.clone();
    let path = price_path(state, &ticker);
    let opened = path(day_ago);
    let mut high = price.price_base_units.max(opened);
    let mut low = price.price_base_units.min(opened);
    for at in (day_ago..price.as_of).step_by(300) {
        let sample = path(at);
        high = high.max(sample);
        low = low.min(sample);
    }
    let mut volume = 0_u64;
    for (at, traded, quantity) in trade_points(state, &instrument.ticker).await? {
        if at >= day_ago {
            high = high.max(traded);
            low = low.min(traded);
            volume += quantity;
        }
    }
    let change = i128::from(price.price_base_units) - i128::from(opened);
    // Same arithmetic as the matching engine's house quote.
    let reference = u128::from(price.price_base_units);
    let spread = u128::from(state.house_half_spread_bps()).min(BPS_DENOMINATOR);
    let bid = reference * (BPS_DENOMINATOR - spread) / BPS_DENOMINATOR;
    let ask = (reference * (BPS_DENOMINATOR + spread)).div_ceil(BPS_DENOMINATOR);
    let quote = quote_asset(&instrument.settlement_currency).to_string();
    Ok(MarketResponse {
        market_id: instrument.market_id,
        symbol: format!("{}/{}", instrument.ticker, quote),
        asset_name: instrument.name,
        base_asset: instrument.ticker,
        quote_asset: quote,
        last_price: display_price(reference, decimals),
        bid: (bid > 0).then(|| display_price(bid, decimals)),
        ask: display_price(ask, decimals),
        status: "OPEN".into(),
        origin: instrument.origin,
        asset_id: instrument.asset_id,
        mint_address: instrument.mint_address,
        settlement_currency: instrument.settlement_currency,
        price_change_24h: market_views::fixed2(change, decimals),
        price_change_percent_24h: market_views::percent(change, opened.into()),
        high_24h: display_price(high.into(), decimals),
        low_24h: display_price(low.into(), decimals),
        volume_24h: volume.to_string(),
        price_source: price.source,
        price_updated_at: price.as_of,
    })
}

pub(crate) async fn list_markets(
    state: &impl Ports,
) -> Result<MarketListResponse, ApplicationError> {
    let catalog = state
        .instrument_provider()
        .catalog()
        .await
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })?;
    let mut data = Vec::with_capacity(catalog.instruments.len());
    for instrument in catalog.instruments {
        data.push(market_response(state, instrument).await?);
    }
    Ok(MarketListResponse { data })
}

pub(crate) async fn get_market(
    state: &impl Ports,
    market_id: String,
) -> Result<MarketDetailResponse, ApplicationError> {
    let instrument = state
        .instrument_provider()
        .catalog()
        .await
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })?
        .instruments
        .into_iter()
        .find(|instrument| instrument.market_id.eq_ignore_ascii_case(&market_id))
        .ok_or_else(|| ApplicationError {
            kind: FailureKind::NotFound,
            error: AuthError::PolicyViolation("market not found".into()),
        })?;
    Ok(MarketDetailResponse {
        data: market_response(state, instrument).await?,
    })
}

/// Engine trades for a ticker, as candle input.
pub(crate) async fn trade_points(
    state: &impl Ports,
    ticker: &str,
) -> Result<Vec<market_views::TradePoint>, ApplicationError> {
    Ok(state
        .market_queries()
        .public_trades(ticker, usize::MAX)
        .await
        .map_err(ApplicationError::from)?
        .into_iter()
        .map(|trade| (trade.executed_at, trade.price_base_units, trade.quantity))
        .collect())
}

pub(crate) async fn market_order_book(
    state: &impl Ports,
    market_id: String,
) -> Result<OrderBookResponse, ApplicationError> {
    let instrument = instrument_by_market(state, &market_id).await?;
    let book = state
        .market_queries()
        .order_book(&instrument.ticker, 15)
        .await
        .map_err(ApplicationError::from)?;
    let decimals = state.price_decimals();
    let side = |levels: Vec<portfolio_trading::BookLevel>| {
        let mut total = 0_u64;
        levels
            .into_iter()
            .map(|level| {
                total += level.quantity;
                OrderBookLevelResponse {
                    price: market_views::fixed2(level.price_base_units.into(), decimals),
                    size: level.quantity.to_string(),
                    total: total.to_string(),
                }
            })
            .collect()
    };
    Ok(OrderBookResponse {
        market_id: instrument.market_id,
        bids: side(book.bids),
        asks: side(book.asks),
        updated_at: market_views::iso(unix_now()),
    })
}

pub(crate) async fn market_trades(
    state: &impl Ports,
    market_id: String,
) -> Result<Vec<PublicTradeResponse>, ApplicationError> {
    let instrument = instrument_by_market(state, &market_id).await?;
    let trades = state
        .market_queries()
        .public_trades(&instrument.ticker, 50)
        .await
        .map_err(ApplicationError::from)?;
    Ok(trades
        .into_iter()
        .map(|trade| PublicTradeResponse {
            trade_id: trade.trade_id,
            market_id: instrument.market_id.clone(),
            price: market_views::fixed2(trade.price_base_units.into(), state.price_decimals()),
            quantity: trade.quantity.to_string(),
            side: match trade.aggressor {
                OrderSide::Buy => "BUY".into(),
                OrderSide::Sell => "SELL".into(),
            },
            executed_at: market_views::iso(trade.executed_at),
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
