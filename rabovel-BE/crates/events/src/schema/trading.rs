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

/// Schema reserved for the colleague's real matching-engine implementation;
/// nothing in this pass publishes it yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecuted {
    pub trade_id: String,
    pub order_id: String,
    pub user_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: String,
    pub price: String,
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
