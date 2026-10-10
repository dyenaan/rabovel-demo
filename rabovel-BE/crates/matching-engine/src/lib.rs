//! Principal-backed matching engine.
//!
//! Matching runs off-chain and is I/O-free and deterministic: callers own
//! persistence, clocks, reference prices and event publication.
//!
//! Every incoming order is executed against the better of two liquidity
//! sources, level by level:
//!
//! 1. **Internalized** — resting client limit orders, in price-time priority.
//! 2. **Principal** — the Rabovel house quote (reference price ± spread),
//!    bounded by per-symbol [`HouseLimits`].
//!
//! On equal prices the book wins, keeping the house flat. Market orders
//! therefore fill instantly whenever the house has a reference price and risk
//! capacity; any remainder is cancelled with an [`UnfilledReason`]. Limit
//! remainders rest and are re-checked against the house on every
//! [`MatchingEngine::reprice`].
//!
//! Principal fills accumulate per symbol until
//! [`MatchingEngine::close_hedge_window`] nets them into clearing-house
//! [`HedgeInstruction`]s (end of day/week, settled T+N). On-chain delivery of
//! each [`Fill`] is the settlement worker's job, not the engine's.

mod book;
mod house;

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use book::{OrderBook, RestingOrder};
use house::HouseState;
pub use house::{
    HedgeInstruction, HedgeSettlement, HouseLimits, HousePosition, HouseQuote, InventoryMovement,
};

/// Whole units of a zero-decimal equity token.
pub type Qty = u64;
/// Payment-asset base units (e.g. cNGN micro-units) per whole unit.
pub type Price = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TradeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }

    /// Whether a counterparty price is acceptable to an order with this side
    /// and limit.
    fn accepts(self, price: Price, limit: Option<Price>) -> bool {
        match (self, limit) {
            (_, None) => true,
            (Side::Buy, Some(limit)) => price <= limit,
            (Side::Sell, Some(limit)) => price >= limit,
        }
    }

    /// Whether `a` is at least as good as `b` for an order with this side.
    fn at_least_as_good(self, a: Price, b: Price) -> bool {
        match self {
            Side::Buy => a <= b,
            Side::Sell => a >= b,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum OrderKind {
    /// Fill immediately at the best available prices; never rests.
    Market,
    /// Fill at `price` or better; the remainder rests on the book.
    Limit { price: Price },
}

impl OrderKind {
    fn limit(self) -> Option<Price> {
        match self {
            OrderKind::Market => None,
            OrderKind::Limit { price } => Some(price),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrder {
    pub account: String,
    pub symbol: String,
    pub side: Side,
    pub kind: OrderKind,
    pub quantity: Qty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Participant {
    Client { account: String, order_id: OrderId },
    House,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiquiditySource {
    /// Client matched against another client's resting order.
    Internalized,
    /// Client matched against the Rabovel house.
    Principal,
}

/// One execution. The seller delivers `quantity` tokens; the buyer delivers
/// `quantity * price` payment-asset base units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fill {
    pub trade_id: TradeId,
    /// Engine-wide, gap-free execution sequence for replay and reconciliation.
    pub sequence: u64,
    pub symbol: String,
    pub buyer: Participant,
    pub seller: Participant,
    pub quantity: Qty,
    pub price: Price,
    pub source: LiquiditySource,
    /// The side that took liquidity.
    pub aggressor: Side,
    /// For principal fills, how the house side moves tokens on-chain.
    pub house_inventory: Option<InventoryMovement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnfilledReason {
    /// The book was empty/unacceptable and the house had no reference price.
    NoReferencePrice,
    /// The house quote side is unavailable (e.g. bid rounded to zero).
    NoHouseQuote,
    /// The house reached its long/short limit for the symbol.
    HouseRiskLimit,
    /// The float is empty and the unbacked-mint limit is reached.
    InventoryLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub order_id: OrderId,
    pub fills: Vec<Fill>,
    pub filled_quantity: Qty,
    pub resting_quantity: Qty,
    pub cancelled_quantity: Qty,
    pub unfilled_reason: Option<UnfilledReason>,
    /// The submitter's own resting orders cancelled by self-trade prevention.
    pub self_trade_cancellations: Vec<OrderId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenOrder {
    pub order_id: OrderId,
    pub account: String,
    pub symbol: String,
    pub side: Side,
    pub price: Price,
    pub original_quantity: Qty,
    pub remaining_quantity: Qty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookDepth {
    pub symbol: String,
    pub bids: Vec<(Price, Qty)>,
    pub asks: Vec<(Price, Qty)>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EngineError {
    #[error("symbol must not be empty")]
    EmptySymbol,
    #[error("quantity must be positive")]
    InvalidQuantity,
    #[error("limit price must be positive")]
    InvalidPrice,
    #[error("order quantity {requested} exceeds the per-order maximum {max}")]
    OrderTooLarge { requested: Qty, max: Qty },
    #[error("order not found")]
    OrderNotFound,
    #[error("hedge sell exceeds the treasury float")]
    InsufficientFloat,
}

#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    pub default_limits: HouseLimits,
    pub symbol_limits: HashMap<String, HouseLimits>,
}

impl EngineConfig {
    pub fn limits(&self, symbol: &str) -> &HouseLimits {
        self.symbol_limits
            .get(symbol)
            .unwrap_or(&self.default_limits)
    }
}

#[derive(Debug, Clone)]
struct OrderLocator {
    account: String,
    symbol: String,
    side: Side,
    price: Price,
}

#[derive(Debug, Default)]
pub struct MatchingEngine {
    config: EngineConfig,
    books: BTreeMap<String, OrderBook>,
    house: BTreeMap<String, HouseState>,
    resting: HashMap<OrderId, OrderLocator>,
    next_order_id: u64,
    next_trade_id: u64,
    next_sequence: u64,
}

impl MatchingEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    /// Executes `order`, using `reference` (if any) to derive the house quote.
    pub fn submit(
        &mut self,
        order: NewOrder,
        reference: Option<Price>,
    ) -> Result<ExecutionReport, EngineError> {
        let limits = *self.config.limits(&order.symbol);
        if order.symbol.trim().is_empty() {
            return Err(EngineError::EmptySymbol);
        }
        if order.quantity == 0 {
            return Err(EngineError::InvalidQuantity);
        }
        if order.kind.limit() == Some(0) {
            return Err(EngineError::InvalidPrice);
        }
        if order.quantity > limits.max_order_quantity {
            return Err(EngineError::OrderTooLarge {
                requested: order.quantity,
                max: limits.max_order_quantity,
            });
        }

        self.next_order_id += 1;
        let order_id = OrderId(self.next_order_id);
        let quote = reference.and_then(|r| HouseQuote::from_reference(r, limits.half_spread_bps));
        let side = order.side;
        let limit = order.kind.limit();
        let mut remaining = order.quantity;
        let mut fills = Vec::new();
        let mut self_trade_cancellations = Vec::new();

        while remaining > 0 {
            let book_price = self
                .books
                .get(&order.symbol)
                .and_then(|book| book.best_price(side.opposite()))
                .filter(|price| side.accepts(*price, limit));
            let house_capacity = self.house_capacity(&order.symbol, side, &limits);
            let house_price = quote
                .and_then(|quote| quote.price_for(side))
                .filter(|price| side.accepts(*price, limit) && house_capacity > 0);

            let use_book = match (book_price, house_price) {
                (Some(book), Some(house)) => side.at_least_as_good(book, house),
                (Some(_), None) => true,
                (None, Some(_)) => false,
                (None, None) => break,
            };

            if use_book {
                let book = self
                    .books
                    .get_mut(&order.symbol)
                    .expect("book has a best price");
                let (price, maker) = book.front_mut(side.opposite()).expect("level is non-empty");
                if maker.account == order.account {
                    let cancelled = book.pop_front(side.opposite()).expect("front exists");
                    self.resting.remove(&cancelled.id);
                    self_trade_cancellations.push(cancelled.id);
                    continue;
                }
                let quantity = remaining.min(maker.remaining);
                maker.remaining -= quantity;
                let maker_participant = Participant::Client {
                    account: maker.account.clone(),
                    order_id: maker.id,
                };
                if maker.remaining == 0 {
                    let done = book.pop_front(side.opposite()).expect("front exists");
                    self.resting.remove(&done.id);
                }
                let taker = Participant::Client {
                    account: order.account.clone(),
                    order_id,
                };
                fills.push(self.make_fill(
                    &order.symbol,
                    side,
                    taker,
                    maker_participant,
                    quantity,
                    price,
                    LiquiditySource::Internalized,
                ));
                remaining -= quantity;
            } else {
                let price = house_price.expect("house was selected");
                let quantity = remaining.min(house_capacity);
                let movement = self
                    .house
                    .entry(order.symbol.clone())
                    .or_default()
                    .record(side, quantity, price);
                let taker = Participant::Client {
                    account: order.account.clone(),
                    order_id,
                };
                let mut fill = self.make_fill(
                    &order.symbol,
                    side,
                    taker,
                    Participant::House,
                    quantity,
                    price,
                    LiquiditySource::Principal,
                );
                fill.house_inventory = Some(movement);
                fills.push(fill);
                remaining -= quantity;
            }
        }

        let filled_quantity = order.quantity - remaining;
        let mut report = ExecutionReport {
            order_id,
            fills,
            filled_quantity,
            resting_quantity: 0,
            cancelled_quantity: 0,
            unfilled_reason: None,
            self_trade_cancellations,
        };
        if remaining == 0 {
            return Ok(report);
        }

        match order.kind {
            OrderKind::Limit { price } => {
                self.books.entry(order.symbol.clone()).or_default().insert(
                    side,
                    price,
                    RestingOrder {
                        id: order_id,
                        account: order.account.clone(),
                        original: order.quantity,
                        remaining,
                    },
                );
                self.resting.insert(
                    order_id,
                    OrderLocator {
                        account: order.account,
                        symbol: order.symbol,
                        side,
                        price,
                    },
                );
                report.resting_quantity = remaining;
            }
            OrderKind::Market => {
                report.cancelled_quantity = remaining;
                report.unfilled_reason = Some(match quote {
                    None => UnfilledReason::NoReferencePrice,
                    Some(quote) if quote.price_for(side).is_none() => UnfilledReason::NoHouseQuote,
                    Some(_) => self.house_exhausted_reason(&order.symbol, side, &limits),
                });
            }
        }
        Ok(report)
    }

    /// Applies a new reference price: resting limit orders that the new house
    /// quote now satisfies are filled by the house at the quote price (never
    /// worse than their limit), best price then oldest first.
    pub fn reprice(&mut self, symbol: &str, reference: Price) -> Vec<Fill> {
        let limits = *self.config.limits(symbol);
        let Some(quote) = HouseQuote::from_reference(reference, limits.half_spread_bps) else {
            return Vec::new();
        };
        let mut fills = Vec::new();
        for client_side in [Side::Buy, Side::Sell] {
            let Some(house_price) = quote.price_for(client_side) else {
                continue;
            };
            loop {
                let capacity = self.house_capacity(symbol, client_side, &limits);
                if capacity == 0 {
                    break;
                }
                let Some(book) = self.books.get_mut(symbol) else {
                    break;
                };
                let Some((limit, maker)) = book.front_mut(client_side) else {
                    break;
                };
                if !client_side.accepts(house_price, Some(limit)) {
                    break;
                }
                let quantity = maker.remaining.min(capacity);
                maker.remaining -= quantity;
                let client = Participant::Client {
                    account: maker.account.clone(),
                    order_id: maker.id,
                };
                if maker.remaining == 0 {
                    let done = book.pop_front(client_side).expect("front exists");
                    self.resting.remove(&done.id);
                }
                let movement = self.house.entry(symbol.to_string()).or_default().record(
                    client_side,
                    quantity,
                    house_price,
                );
                // The house re-quoted into the resting order, so it is the aggressor.
                let mut fill = self.make_fill(
                    symbol,
                    client_side.opposite(),
                    Participant::House,
                    client,
                    quantity,
                    house_price,
                    LiquiditySource::Principal,
                );
                fill.house_inventory = Some(movement);
                fills.push(fill);
            }
        }
        fills
    }

    /// Cancels a resting order. Orders owned by other accounts are reported as
    /// not found so their existence is not disclosed.
    pub fn cancel(&mut self, account: &str, order_id: OrderId) -> Result<OpenOrder, EngineError> {
        let locator = self
            .resting
            .get(&order_id)
            .filter(|locator| locator.account == account)
            .cloned()
            .ok_or(EngineError::OrderNotFound)?;
        let removed = self
            .books
            .get_mut(&locator.symbol)
            .and_then(|book| book.remove(locator.side, locator.price, order_id))
            .ok_or(EngineError::OrderNotFound)?;
        self.resting.remove(&order_id);
        Ok(OpenOrder {
            order_id,
            account: locator.account,
            symbol: locator.symbol,
            side: locator.side,
            price: locator.price,
            original_quantity: removed.original,
            remaining_quantity: removed.remaining,
        })
    }

    /// Resting orders for `account`, ordered by order id.
    pub fn open_orders(&self, account: &str) -> Vec<OpenOrder> {
        let mut orders: Vec<OpenOrder> = self
            .resting
            .iter()
            .filter(|(_, locator)| locator.account == account)
            .filter_map(|(id, locator)| {
                let resting =
                    self.books
                        .get(&locator.symbol)?
                        .get(locator.side, locator.price, *id)?;
                Some(OpenOrder {
                    order_id: *id,
                    account: locator.account.clone(),
                    symbol: locator.symbol.clone(),
                    side: locator.side,
                    price: locator.price,
                    original_quantity: resting.original,
                    remaining_quantity: resting.remaining,
                })
            })
            .collect();
        orders.sort_by_key(|order| order.order_id);
        orders
    }

    pub fn depth(&self, symbol: &str, max_levels: usize) -> BookDepth {
        let book = self.books.get(symbol);
        BookDepth {
            symbol: symbol.to_string(),
            bids: book.map_or_else(Vec::new, |b| b.depth(Side::Buy, max_levels)),
            asks: book.map_or_else(Vec::new, |b| b.depth(Side::Sell, max_levels)),
        }
    }

    pub fn house_position(&self, symbol: &str) -> HousePosition {
        self.house
            .get(symbol)
            .map(|state| state.position.clone())
            .unwrap_or_default()
    }

    /// Adds backed tokens to the house treasury float for `symbol` (issuer
    /// inventory transferred in, or pre-funding ahead of trading).
    pub fn fund_float(&mut self, symbol: &str, quantity: Qty) {
        self.house
            .entry(symbol.to_string())
            .or_default()
            .fund_float(quantity);
    }

    /// Records a clearing-house hedge settling (T+N). A buy backs unbacked
    /// mints first, then refills the float; a sell must not exceed the float.
    pub fn settle_hedge(
        &mut self,
        symbol: &str,
        hedge_side: Side,
        quantity: Qty,
    ) -> Result<HedgeSettlement, EngineError> {
        if quantity == 0 {
            return Err(EngineError::InvalidQuantity);
        }
        self.house
            .entry(symbol.to_string())
            .or_default()
            .settle_hedge(hedge_side, quantity)
            .ok_or(EngineError::InsufficientFloat)
    }

    /// Nets all principal fills since the previous close into one instruction
    /// per symbol (sorted by symbol) and starts a new window. Positions persist.
    pub fn close_hedge_window(&mut self) -> Vec<HedgeInstruction> {
        self.house
            .iter_mut()
            .filter_map(|(symbol, state)| state.close_window(symbol))
            .collect()
    }

    fn house_capacity(&self, symbol: &str, client_side: Side, limits: &HouseLimits) -> Qty {
        self.house.get(symbol).map_or_else(
            || HouseState::default().capacity(client_side, limits),
            |state| state.capacity(client_side, limits),
        )
    }

    fn house_exhausted_reason(
        &self,
        symbol: &str,
        client_side: Side,
        limits: &HouseLimits,
    ) -> UnfilledReason {
        self.house.get(symbol).map_or_else(
            || HouseState::default().exhausted_reason(client_side, limits),
            |state| state.exhausted_reason(client_side, limits),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn make_fill(
        &mut self,
        symbol: &str,
        aggressor: Side,
        aggressor_participant: Participant,
        passive_participant: Participant,
        quantity: Qty,
        price: Price,
        source: LiquiditySource,
    ) -> Fill {
        self.next_trade_id += 1;
        self.next_sequence += 1;
        let (buyer, seller) = match aggressor {
            Side::Buy => (aggressor_participant, passive_participant),
            Side::Sell => (passive_participant, aggressor_participant),
        };
        Fill {
            trade_id: TradeId(self.next_trade_id),
            sequence: self.next_sequence,
            symbol: symbol.to_string(),
            buyer,
            seller,
            quantity,
            price,
            source,
            aggressor,
            house_inventory: None,
        }
    }
}

#[cfg(test)]
mod tests;
