use crate::house::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

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
    pub(super) fn accepts(self, price: Price, limit: Option<Price>) -> bool {
        match (self, limit) {
            (_, None) => true,
            (Side::Buy, Some(limit)) => price <= limit,
            (Side::Sell, Some(limit)) => price >= limit,
        }
    }

    /// Whether `a` is at least as good as `b` for an order with this side.
    pub(super) fn at_least_as_good(self, a: Price, b: Price) -> bool {
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
    pub(super) fn limit(self) -> Option<Price> {
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
pub(super) struct OrderLocator {
    pub(super) account: String,
    pub(super) symbol: String,
    pub(super) side: Side,
    pub(super) price: Price,
}
