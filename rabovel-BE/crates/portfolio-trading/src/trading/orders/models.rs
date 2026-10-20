use crate::*;

/// One aggregated price level, in base units.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BookLevel {
    pub price_base_units: u64,
    pub quantity: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<BookLevel>,
    pub asks: Vec<BookLevel>,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PublicTrade {
    pub trade_id: String,
    pub symbol: String,
    pub price_base_units: u64,
    pub quantity: u64,
    /// Side of the order that took liquidity.
    pub aggressor: OrderSide,
    pub executed_at: u64,
}
/// Which side of the market an order is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    Buy,
    Sell,
}
/// The order execution style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
}
/// A request to place a new order. `quantity` is in whole units;
/// `limit_price` is in payment-asset units (e.g. `"100.25"` cNGN).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NewOrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: String,
    pub limit_price: Option<String>,
}
/// Where a fill's liquidity came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Liquidity {
    /// Matched against another client's resting order.
    Internalized,
    /// Matched against the Rabovel house.
    Principal,
}
/// One execution of the caller's order, without counterparty identity.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OrderFill {
    pub trade_id: String,
    pub quantity: String,
    pub price: String,
    pub liquidity: Liquidity,
}
/// Acknowledgement returned after an order is accepted and executed.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OrderAck {
    pub order_id: String,
    pub accepted_at: u64,
    pub state: OrderState,
    pub filled_quantity: String,
    /// Quantity left resting on the book (limit orders).
    pub resting_quantity: String,
    /// Quantity that could not be filled and was dropped (market orders).
    pub cancelled_quantity: String,
    pub average_price: Option<String>,
    pub fills: Vec<OrderFill>,
    /// Why `cancelled_quantity` was not filled, e.g. `house_risk_limit`.
    pub unfilled_reason: Option<String>,
}
/// A party to an executed trade.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TradeParty {
    Client { user_id: String, order_id: String },
    House,
}
/// Internal record of one execution, for settlement and reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedTrade {
    pub trade_id: String,
    /// Gap-free engine sequence, for replay and ordering.
    pub sequence: u64,
    pub symbol: String,
    /// Whole token units.
    pub quantity: u64,
    /// Payment-asset base units per token unit.
    pub price_base_units: u64,
    pub price_decimals: u8,
    pub buyer: TradeParty,
    pub seller: TradeParty,
    pub liquidity: Liquidity,
    pub aggressor: OrderSide,
    /// For principal trades, how the house side moves tokens on-chain.
    pub house_inventory: Option<InventoryMovement>,
    pub executed_at: u64,
}
/// Result of placing an order.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderExecution {
    pub ack: OrderAck,
    /// Every trade this request caused, including fills of other clients'
    /// resting orders triggered by a reference-price update.
    pub trades: Vec<ExecutedTrade>,
}
/// The lifecycle state of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderState {
    /// Resting on the book with nothing filled.
    Accepted,
    PartiallyFilled,
    Filled,
    /// Cancelled by the client, dropped unfilled (market remainder) or
    /// cancelled by self-trade prevention; may carry a partial fill.
    Cancelled,
}
/// The current status of an order.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OrderStatus {
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub limit_price: Option<String>,
    pub quantity: String,
    pub filled_quantity: String,
    pub remaining_quantity: String,
    pub average_price: Option<String>,
    pub state: OrderState,
    pub created_at: u64,
    pub updated_at: u64,
}
/// Errors that can occur while interacting with the portfolio trading API.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TradingError {
    #[error("order not found")]
    OrderNotFound,
    #[error("invalid quantity: {0}")]
    InvalidQuantity(String),
    #[error("invalid limit price: {0}")]
    InvalidPrice(String),
    #[error("invalid order: {0}")]
    InvalidOrder(String),
    #[error("unknown symbol: {0}")]
    UnknownSymbol(String),
    #[error("order too large: {0}")]
    OrderTooLarge(String),
    #[error("service unavailable: {0}")]
    Unavailable(String),
}
