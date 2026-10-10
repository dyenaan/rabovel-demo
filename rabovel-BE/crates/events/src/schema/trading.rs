use serde::{Deserialize, Serialize};

use crate::{envelope::DomainEvent, topics};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAccepted {
    pub order_id: String,
    pub user_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: String,
    pub accepted_at: u64,
}

impl DomainEvent for OrderAccepted {
    const EVENT_TYPE: &'static str = "trading.order_accepted";
    const TOPIC: &'static str = topics::TRADING_ORDER_ACCEPTED_V1;
}

/// A party to an executed trade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TradeParty {
    Client {
        user_id: String,
        order_id: String,
    },
    /// The Rabovel principal desk.
    House,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeLiquidity {
    /// Client matched against another client's resting order.
    Internalized,
    /// Client matched against the house.
    Principal,
}

/// How the house side of a principal trade moves tokens on-chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum HouseInventory {
    /// Transfer `from_float` from the treasury; mint `minted` against the
    /// pending clearing-house hedge.
    HouseDelivers { from_float: u64, minted: u64 },
    /// Burn `retired` (cancelling unbacked supply); keep `to_float` in the
    /// treasury.
    HouseReceives { to_float: u64, retired: u64 },
}

/// One execution from the matching engine, published once per fill. The
/// seller delivers `quantity` tokens; the buyer delivers
/// `quantity * price_base_units` payment-asset base units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeExecuted {
    pub trade_id: String,
    /// Gap-free engine sequence for ordering and replay.
    pub sequence: u64,
    pub symbol: String,
    /// Whole token units.
    pub quantity: String,
    pub price_base_units: String,
    pub price_decimals: u8,
    pub buyer: TradeParty,
    pub seller: TradeParty,
    pub liquidity: TradeLiquidity,
    pub aggressor: OrderSide,
    pub house_inventory: Option<HouseInventory>,
    pub executed_at: u64,
}

impl DomainEvent for TradeExecuted {
    const EVENT_TYPE: &'static str = "trading.trade_executed";
    const TOPIC: &'static str = topics::TRADING_TRADE_EXECUTED_V1;
}

/// Schema reserved; not published this pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioChanged {
    pub user_id: String,
    pub asset: String,
    pub total: String,
    pub available: String,
    pub changed_at: u64,
}

impl DomainEvent for PortfolioChanged {
    const EVENT_TYPE: &'static str = "portfolio.changed";
    const TOPIC: &'static str = topics::PORTFOLIO_CHANGED_V1;
}
