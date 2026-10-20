use super::*;
use crate::now_secs;
use crate::trading::execution::decimal::format_signed;
use crate::AssetBalance;
use crate::BookLevel;
use crate::OrderBook;
use crate::OrderStatus;
use crate::PortfolioSnapshot;
use crate::PublicTrade;
use crate::TradingError;
use matching_engine::HouseQuote;
use matching_engine::Price;
use matching_engine::Qty;
use matching_engine::Side;
use std::collections::BTreeMap;

#[async_trait::async_trait]
impl crate::trading::orders::ports::OrderQueries for EngineTradingService {
    async fn list_orders(&self, user_id: &str) -> Result<Vec<OrderStatus>, TradingError> {
        let state = self.lock()?;
        let ids = state.user_orders.get(user_id).cloned().unwrap_or_default();
        // Newest first.
        Ok(ids
            .into_iter()
            .rev()
            .filter_map(|id| {
                let record = state.orders.get(&id)?;
                Some(self.order_status(id, record, state.remaining(user_id, id)))
            })
            .collect())
    }
    async fn get_order(&self, user_id: &str, order_id: &str) -> Result<OrderStatus, TradingError> {
        let state = self.lock()?;
        let id = state.owned_order(user_id, order_id)?;
        let record = state.orders.get(&id).ok_or(TradingError::OrderNotFound)?;
        Ok(self.order_status(id, record, state.remaining(user_id, id)))
    }
}
#[async_trait::async_trait]
impl crate::portfolio::balances::ports::BalanceQueries for EngineTradingService {
    async fn get_portfolio(&self, user_id: &str) -> Result<PortfolioSnapshot, TradingError> {
        let decimals = self.config.price_decimals;
        let state = self.lock()?;
        let mut totals = state.balances.get(user_id).cloned().unwrap_or_default();
        let mut locked: BTreeMap<String, i128> = BTreeMap::new();
        for open in state.engine.open_orders(user_id) {
            let (asset, amount) = match open.side {
                Side::Sell => (open.symbol, i128::from(open.remaining_quantity)),
                Side::Buy => (
                    String::new(),
                    i128::from(open.remaining_quantity) * i128::from(open.price),
                ),
            };
            *locked.entry(asset.clone()).or_default() += amount;
            totals.entry(asset).or_default();
        }
        let balances = totals
            .into_iter()
            .map(|(asset, total)| {
                let locked = locked.get(&asset).copied().unwrap_or_default();
                let scale = if asset.is_empty() { decimals } else { 0 };
                AssetBalance {
                    asset: if asset.is_empty() {
                        self.config.payment_asset.clone()
                    } else {
                        asset
                    },
                    total: format_signed(total, scale),
                    available: format_signed(total - locked, scale),
                    locked: format_signed(locked, scale),
                }
            })
            .collect();
        Ok(PortfolioSnapshot {
            user_id: user_id.to_string(),
            balances,
        })
    }
}
#[async_trait::async_trait]
impl crate::trading::market_queries::ports::MarketQueries for EngineTradingService {
    async fn order_book(&self, symbol: &str, levels: usize) -> Result<OrderBook, TradingError> {
        let symbol = normalize_symbol(symbol);
        let reference = self
            .prices
            .reference_price(&symbol, self.config.price_decimals)
            .await
            .map_err(TradingError::Unavailable)?
            .ok_or_else(|| TradingError::UnknownSymbol(symbol.clone()))?;
        let fresh = now_secs().saturating_sub(reference.as_of) <= self.config.max_price_age_secs;

        let state = self.lock()?;
        let depth = state.engine.depth(&symbol, levels);
        let mut bids: BTreeMap<Price, Qty> = depth.bids.into_iter().collect();
        let mut asks: BTreeMap<Price, Qty> = depth.asks.into_iter().collect();
        let limits = state.engine.limits(&symbol);
        if let Some(quote) = fresh
            .then(|| HouseQuote::from_reference(reference.price_base_units, limits.half_spread_bps))
            .flatten()
        {
            // The house sells at its ask to client buys and buys at its bid
            // from client sells; one order is capped by max_order_quantity.
            let ask_size = state
                .engine
                .house_quote_capacity(&symbol, Side::Buy)
                .min(limits.max_order_quantity);
            if ask_size > 0 {
                *asks.entry(quote.ask).or_default() += ask_size;
            }
            let bid_size = state
                .engine
                .house_quote_capacity(&symbol, Side::Sell)
                .min(limits.max_order_quantity);
            if let (Some(bid), true) = (quote.bid, bid_size > 0) {
                *bids.entry(bid).or_default() += bid_size;
            }
        }
        let level = |(price, quantity): (Price, Qty)| BookLevel {
            price_base_units: price,
            quantity,
        };
        Ok(OrderBook {
            symbol,
            bids: bids.into_iter().rev().take(levels).map(level).collect(),
            asks: asks.into_iter().take(levels).map(level).collect(),
        })
    }
    async fn public_trades(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<PublicTrade>, TradingError> {
        let state = self.lock()?;
        Ok(state
            .public_trades
            .get(&normalize_symbol(symbol))
            .map(|tape| tape.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default())
    }
}
