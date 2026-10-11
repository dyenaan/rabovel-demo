//! [`PortfolioTradingApi`] backed by the principal matching engine.
//!
//! State (engine, order records, net balances) is process-local and lost on
//! restart; durable journaling is a later slice. Balances are the net effect
//! of trades matched here, not on-chain holdings, and orders are not yet
//! checked against buying power or token balance.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};

use matching_engine::{
    EngineConfig, EngineError, Fill, HouseQuote, LiquiditySource, MatchingEngine, NewOrder,
    OrderId, OrderKind, Participant, Price, Qty, Side, UnfilledReason,
};

use crate::decimal::{format_fixed, format_signed, parse_fixed};
use crate::{
    now_secs, AssetBalance, BookLevel, ExecutedTrade, Liquidity, NewOrderRequest, OrderAck,
    OrderBook, OrderExecution, OrderFill, OrderSide, OrderState, OrderStatus, OrderType,
    PortfolioSnapshot, PortfolioTradingApi, PublicTrade, TradeParty, TradingError,
};

/// Recent public trades kept per symbol for the tape and price history.
const MAX_PUBLIC_TRADES: usize = 1_000;

/// A reference price in payment-asset base units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferencePrice {
    pub price_base_units: u64,
    /// Unix seconds the price was observed.
    pub as_of: u64,
}

/// Port for tradable instruments and their reference prices.
#[async_trait::async_trait]
pub trait ReferencePriceSource: Send + Sync {
    /// `Ok(None)` when `symbol` is not a tradable instrument; `Err` when the
    /// source cannot answer.
    async fn reference_price(
        &self,
        symbol: &str,
        price_decimals: u8,
    ) -> Result<Option<ReferencePrice>, String>;
}

#[derive(Debug, Clone)]
pub struct EngineTradingConfig {
    pub engine: EngineConfig,
    /// Decimals of the payment asset prices are quoted in.
    pub price_decimals: u8,
    /// Balance name of the payment asset.
    pub payment_asset: String,
    /// Older reference prices are ignored: the house stops quoting and only
    /// client-to-client matching remains.
    pub max_price_age_secs: u64,
    /// Treasury float pre-funded per symbol at startup.
    pub initial_float: HashMap<String, Qty>,
}

impl Default for EngineTradingConfig {
    fn default() -> Self {
        Self {
            engine: EngineConfig::default(),
            price_decimals: 6,
            payment_asset: "CNGN".into(),
            max_price_age_secs: 60,
            initial_float: HashMap::new(),
        }
    }
}

pub struct EngineTradingService {
    prices: Arc<dyn ReferencePriceSource>,
    config: EngineTradingConfig,
    state: Mutex<TradingState>,
}

struct OrderRecord {
    user_id: String,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: Qty,
    limit_price: Option<Price>,
    filled: Qty,
    notional: u128,
    state: OrderState,
    created_at: u64,
    updated_at: u64,
}

#[derive(Default)]
struct TradingState {
    engine: MatchingEngine,
    orders: HashMap<OrderId, OrderRecord>,
    user_orders: HashMap<String, Vec<OrderId>>,
    /// user → asset → net base units (tokens are whole units).
    balances: HashMap<String, BTreeMap<String, i128>>,
    /// symbol → recent trades, newest at the back.
    public_trades: HashMap<String, VecDeque<PublicTrade>>,
}

impl EngineTradingService {
    pub fn new(prices: Arc<dyn ReferencePriceSource>, config: EngineTradingConfig) -> Self {
        let mut engine = MatchingEngine::new(config.engine.clone());
        for (symbol, quantity) in &config.initial_float {
            engine.fund_float(&normalize_symbol(symbol), *quantity);
        }
        Self {
            prices,
            config,
            state: Mutex::new(TradingState {
                engine,
                ..TradingState::default()
            }),
        }
    }

    fn lock(&self) -> Result<MutexGuard<'_, TradingState>, TradingError> {
        self.state
            .lock()
            .map_err(|_| TradingError::Unavailable("trading state is unavailable".into()))
    }

    fn order_status(&self, id: OrderId, record: &OrderRecord, remaining: Qty) -> OrderStatus {
        let decimals = self.config.price_decimals;
        OrderStatus {
            order_id: format_order_id(id),
            symbol: record.symbol.clone(),
            side: record.side,
            order_type: record.order_type,
            limit_price: record
                .limit_price
                .map(|price| format_fixed(price.into(), decimals)),
            quantity: record.quantity.to_string(),
            filled_quantity: record.filled.to_string(),
            remaining_quantity: remaining.to_string(),
            average_price: average_price(record.notional, record.filled, decimals),
            state: record.state,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

impl TradingState {
    /// Applies fills to order records and balances, returning trade records.
    fn apply_fills(&mut self, fills: &[Fill], now: u64, price_decimals: u8) -> Vec<ExecutedTrade> {
        fills
            .iter()
            .map(|fill| {
                let notional = u128::from(fill.quantity) * u128::from(fill.price);
                for (participant, side) in [(&fill.buyer, Side::Buy), (&fill.seller, Side::Sell)] {
                    let Participant::Client { account, order_id } = participant else {
                        continue;
                    };
                    if let Some(record) = self.orders.get_mut(order_id) {
                        record.filled += fill.quantity;
                        record.notional += notional;
                        record.state = if record.filled == record.quantity {
                            OrderState::Filled
                        } else {
                            OrderState::PartiallyFilled
                        };
                        record.updated_at = now;
                    }
                    let balances = self.balances.entry(account.clone()).or_default();
                    let signed_quantity = i128::from(fill.quantity);
                    let signed_notional = notional as i128;
                    let (asset_delta, cash_delta) = match side {
                        Side::Buy => (signed_quantity, -signed_notional),
                        Side::Sell => (-signed_quantity, signed_notional),
                    };
                    *balances.entry(fill.symbol.clone()).or_default() += asset_delta;
                    *balances.entry(String::new()).or_default() += cash_delta;
                }
                let tape = self.public_trades.entry(fill.symbol.clone()).or_default();
                if tape.len() == MAX_PUBLIC_TRADES {
                    tape.pop_front();
                }
                tape.push_back(PublicTrade {
                    trade_id: format!("trd_{}", fill.trade_id.0),
                    symbol: fill.symbol.clone(),
                    price_base_units: fill.price,
                    quantity: fill.quantity,
                    aggressor: from_engine_side(fill.aggressor),
                    executed_at: now,
                });
                ExecutedTrade {
                    trade_id: format!("trd_{}", fill.trade_id.0),
                    sequence: fill.sequence,
                    symbol: fill.symbol.clone(),
                    quantity: fill.quantity,
                    price_base_units: fill.price,
                    price_decimals,
                    buyer: trade_party(&fill.buyer),
                    seller: trade_party(&fill.seller),
                    liquidity: liquidity(fill.source),
                    aggressor: from_engine_side(fill.aggressor),
                    house_inventory: fill.house_inventory,
                    executed_at: now,
                }
            })
            .collect()
    }

    fn mark_cancelled(&mut self, id: OrderId, now: u64) {
        if let Some(record) = self.orders.get_mut(&id) {
            record.state = OrderState::Cancelled;
            record.updated_at = now;
        }
    }

    fn owned_order(&self, user_id: &str, order_id: &str) -> Result<OrderId, TradingError> {
        parse_order_id(order_id)
            .filter(|id| {
                self.orders
                    .get(id)
                    .is_some_and(|record| record.user_id == user_id)
            })
            .ok_or(TradingError::OrderNotFound)
    }

    fn remaining(&self, user_id: &str, id: OrderId) -> Qty {
        self.engine
            .open_orders(user_id)
            .into_iter()
            .find(|open| open.order_id == id)
            .map_or(0, |open| open.remaining_quantity)
    }
}

#[async_trait::async_trait]
impl PortfolioTradingApi for EngineTradingService {
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

    async fn place_order(
        &self,
        user_id: &str,
        order: NewOrderRequest,
    ) -> Result<OrderExecution, TradingError> {
        let decimals = self.config.price_decimals;
        let symbol = normalize_symbol(&order.symbol);
        if symbol.is_empty() {
            return Err(TradingError::UnknownSymbol(order.symbol));
        }
        let quantity = parse_fixed(&order.quantity, 0)
            .filter(|quantity| *quantity > 0)
            .ok_or_else(|| TradingError::InvalidQuantity(order.quantity.clone()))?;
        let (kind, limit_price) = match (order.order_type, order.limit_price.as_deref()) {
            (OrderType::Market, None) => (OrderKind::Market, None),
            (OrderType::Market, Some(_)) => {
                return Err(TradingError::InvalidOrder(
                    "market orders do not take a limit price".into(),
                ))
            }
            (OrderType::Limit, Some(raw)) => {
                let price = parse_fixed(raw, decimals)
                    .filter(|price| *price > 0)
                    .ok_or_else(|| TradingError::InvalidPrice(raw.to_string()))?;
                (OrderKind::Limit { price }, Some(price))
            }
            (OrderType::Limit, None) => {
                return Err(TradingError::InvalidOrder(
                    "limit orders require a limit price".into(),
                ))
            }
        };
        let reference = self
            .prices
            .reference_price(&symbol, decimals)
            .await
            .map_err(TradingError::Unavailable)?
            .ok_or_else(|| TradingError::UnknownSymbol(symbol.clone()))?;

        let now = now_secs();
        let fresh = (now.saturating_sub(reference.as_of) <= self.config.max_price_age_secs)
            .then_some(reference.price_base_units);

        let mut state = self.lock()?;
        let mut trades = Vec::new();
        if let Some(price) = fresh {
            // Fill resting limits the current quote already satisfies, so the
            // new order never trades ahead of them.
            let fills = state.engine.reprice(&symbol, price);
            trades.extend(state.apply_fills(&fills, now, decimals));
        }

        let report = state
            .engine
            .submit(
                NewOrder {
                    account: user_id.to_string(),
                    symbol: symbol.clone(),
                    side: to_engine_side(order.side),
                    kind,
                    quantity,
                },
                fresh,
            )
            .map_err(map_engine_error)?;

        state.orders.insert(
            report.order_id,
            OrderRecord {
                user_id: user_id.to_string(),
                symbol,
                side: order.side,
                order_type: order.order_type,
                quantity,
                limit_price,
                filled: 0,
                notional: 0,
                state: OrderState::Accepted,
                created_at: now,
                updated_at: now,
            },
        );
        state
            .user_orders
            .entry(user_id.to_string())
            .or_default()
            .push(report.order_id);
        for cancelled in &report.self_trade_cancellations {
            state.mark_cancelled(*cancelled, now);
        }
        trades.extend(state.apply_fills(&report.fills, now, decimals));

        let record = state
            .orders
            .get_mut(&report.order_id)
            .expect("order was just inserted");
        record.state = if report.resting_quantity > 0 {
            if report.filled_quantity > 0 {
                OrderState::PartiallyFilled
            } else {
                OrderState::Accepted
            }
        } else if report.cancelled_quantity > 0 {
            OrderState::Cancelled
        } else {
            OrderState::Filled
        };

        let ack = OrderAck {
            order_id: format_order_id(report.order_id),
            accepted_at: now,
            state: record.state,
            filled_quantity: report.filled_quantity.to_string(),
            resting_quantity: report.resting_quantity.to_string(),
            cancelled_quantity: report.cancelled_quantity.to_string(),
            average_price: average_price(record.notional, record.filled, decimals),
            fills: report
                .fills
                .iter()
                .map(|fill| OrderFill {
                    trade_id: format!("trd_{}", fill.trade_id.0),
                    quantity: fill.quantity.to_string(),
                    price: format_fixed(fill.price.into(), decimals),
                    liquidity: liquidity(fill.source),
                })
                .collect(),
            unfilled_reason: report
                .unfilled_reason
                .map(|reason| unfilled_reason(reason).into()),
        };
        Ok(OrderExecution { ack, trades })
    }

    async fn cancel_order(&self, user_id: &str, order_id: &str) -> Result<(), TradingError> {
        let mut state = self.lock()?;
        let id = state.owned_order(user_id, order_id)?;
        state.engine.cancel(user_id, id).map_err(map_engine_error)?;
        state.mark_cancelled(id, now_secs());
        Ok(())
    }

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

    async fn reprice_resting(&self) -> Result<Vec<ExecutedTrade>, TradingError> {
        let decimals = self.config.price_decimals;
        let symbols = self.lock()?.engine.symbols_with_resting_orders();
        // Fetch prices without holding the lock across awaits.
        let mut prices = Vec::new();
        for symbol in symbols {
            let reference = self
                .prices
                .reference_price(&symbol, decimals)
                .await
                .map_err(TradingError::Unavailable)?;
            if let Some(reference) = reference.filter(|reference| {
                now_secs().saturating_sub(reference.as_of) <= self.config.max_price_age_secs
            }) {
                prices.push((symbol, reference.price_base_units));
            }
        }
        let now = now_secs();
        let mut state = self.lock()?;
        let mut trades = Vec::new();
        for (symbol, price) in prices {
            let fills = state.engine.reprice(&symbol, price);
            trades.extend(state.apply_fills(&fills, now, decimals));
        }
        Ok(trades)
    }

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

fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_ascii_uppercase()
}

fn format_order_id(id: OrderId) -> String {
    format!("ord_{}", id.0)
}

fn parse_order_id(order_id: &str) -> Option<OrderId> {
    order_id.strip_prefix("ord_")?.parse().ok().map(OrderId)
}

fn average_price(notional: u128, filled: Qty, decimals: u8) -> Option<String> {
    (filled > 0).then(|| format_fixed(notional / u128::from(filled), decimals))
}

fn to_engine_side(side: OrderSide) -> Side {
    match side {
        OrderSide::Buy => Side::Buy,
        OrderSide::Sell => Side::Sell,
    }
}

fn from_engine_side(side: Side) -> OrderSide {
    match side {
        Side::Buy => OrderSide::Buy,
        Side::Sell => OrderSide::Sell,
    }
}

fn liquidity(source: LiquiditySource) -> Liquidity {
    match source {
        LiquiditySource::Internalized => Liquidity::Internalized,
        LiquiditySource::Principal => Liquidity::Principal,
    }
}

fn trade_party(participant: &Participant) -> TradeParty {
    match participant {
        Participant::Client { account, order_id } => TradeParty::Client {
            user_id: account.clone(),
            order_id: format_order_id(*order_id),
        },
        Participant::House => TradeParty::House,
    }
}

fn unfilled_reason(reason: UnfilledReason) -> &'static str {
    match reason {
        UnfilledReason::NoReferencePrice => "no_reference_price",
        UnfilledReason::NoHouseQuote => "no_house_quote",
        UnfilledReason::HouseRiskLimit => "house_risk_limit",
        UnfilledReason::InventoryLimit => "inventory_limit",
    }
}

fn map_engine_error(error: EngineError) -> TradingError {
    match error {
        EngineError::EmptySymbol => TradingError::UnknownSymbol(String::new()),
        EngineError::InvalidQuantity => TradingError::InvalidQuantity(error.to_string()),
        EngineError::InvalidPrice => TradingError::InvalidPrice(error.to_string()),
        EngineError::OrderTooLarge { .. } => TradingError::OrderTooLarge(error.to_string()),
        EngineError::OrderNotFound => TradingError::OrderNotFound,
        EngineError::InsufficientFloat => TradingError::Unavailable(error.to_string()),
    }
}

#[cfg(test)]
mod tests;
