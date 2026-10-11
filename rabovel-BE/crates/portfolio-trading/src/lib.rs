//! Portfolio trading API contract and its implementations.
//!
//! [`PortfolioTradingApi`] is what the gateway calls. [`EngineTradingService`]
//! is the real implementation, backed by the principal matching engine in
//! the `matching-engine` crate; [`StubPortfolioTradingService`] is a
//! deterministic placeholder kept for tests that do not exercise trading.
//! Amounts cross this boundary as decimal strings and are converted exactly to
//! integer base units, never through floats.

mod decimal;
mod engine_service;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub use engine_service::{
    EngineTradingConfig, EngineTradingService, ReferencePrice, ReferencePriceSource,
};
pub use matching_engine::{EngineConfig, HouseLimits, InventoryMovement};

/// The portfolio trading API contract.
#[async_trait::async_trait]
pub trait PortfolioTradingApi: Send + Sync {
    async fn get_portfolio(&self, user_id: &str) -> Result<PortfolioSnapshot, TradingError>;
    /// Executes the order immediately as far as liquidity allows. The returned
    /// [`OrderExecution::trades`] are for event publication and must not be
    /// returned to the client: they identify counterparties.
    async fn place_order(
        &self,
        user_id: &str,
        order: NewOrderRequest,
    ) -> Result<OrderExecution, TradingError>;
    async fn cancel_order(&self, user_id: &str, order_id: &str) -> Result<(), TradingError>;
    async fn list_orders(&self, user_id: &str) -> Result<Vec<OrderStatus>, TradingError>;
    async fn get_order(&self, user_id: &str, order_id: &str) -> Result<OrderStatus, TradingError>;

    /// Lets the house fill resting limit orders that the current reference
    /// prices now satisfy. Called on a timer so limits fill as the market
    /// moves, not only when someone else trades the symbol. Returns the
    /// trades for event publication.
    async fn reprice_resting(&self) -> Result<Vec<ExecutedTrade>, TradingError> {
        Ok(Vec::new())
    }

    /// Aggregated public book for `symbol`, best price first: resting client
    /// orders plus the house quote when its reference price is fresh.
    async fn order_book(&self, symbol: &str, levels: usize) -> Result<OrderBook, TradingError> {
        let _ = levels;
        Ok(OrderBook {
            symbol: symbol.to_ascii_uppercase(),
            bids: Vec::new(),
            asks: Vec::new(),
        })
    }

    /// Most recent matched trades for `symbol`, newest first. Anonymous: no
    /// order ids or counterparties.
    async fn public_trades(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<PublicTrade>, TradingError> {
        let _ = (symbol, limit);
        Ok(Vec::new())
    }
}

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

/// A snapshot of a user's portfolio at a point in time.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortfolioSnapshot {
    pub user_id: String,
    pub balances: Vec<AssetBalance>,
}

/// A single asset's balance breakdown.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetBalance {
    pub asset: String,
    pub total: String,
    pub available: String,
    pub locked: String,
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

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// A deterministic placeholder implementation of [`PortfolioTradingApi`].
///
/// Orders are acknowledged but never matched or persisted.
#[derive(Debug, Default)]
pub struct StubPortfolioTradingService {
    next_order_id: AtomicU64,
}

#[async_trait::async_trait]
impl PortfolioTradingApi for StubPortfolioTradingService {
    async fn get_portfolio(&self, user_id: &str) -> Result<PortfolioSnapshot, TradingError> {
        Ok(PortfolioSnapshot {
            user_id: user_id.to_string(),
            balances: vec![],
        })
    }

    async fn place_order(
        &self,
        _user_id: &str,
        order: NewOrderRequest,
    ) -> Result<OrderExecution, TradingError> {
        decimal::parse_fixed(&order.quantity, 0)
            .filter(|quantity| *quantity > 0)
            .ok_or_else(|| TradingError::InvalidQuantity(order.quantity.clone()))?;

        let n = self.next_order_id.fetch_add(1, Ordering::SeqCst);
        Ok(OrderExecution {
            ack: OrderAck {
                order_id: format!("ord_{n}"),
                accepted_at: now_secs(),
                state: OrderState::Accepted,
                filled_quantity: "0".into(),
                resting_quantity: order.quantity,
                cancelled_quantity: "0".into(),
                average_price: None,
                fills: vec![],
                unfilled_reason: None,
            },
            trades: vec![],
        })
    }

    async fn cancel_order(&self, _user_id: &str, _order_id: &str) -> Result<(), TradingError> {
        Ok(())
    }

    async fn list_orders(&self, _user_id: &str) -> Result<Vec<OrderStatus>, TradingError> {
        Ok(vec![])
    }

    async fn get_order(
        &self,
        _user_id: &str,
        _order_id: &str,
    ) -> Result<OrderStatus, TradingError> {
        Err(TradingError::OrderNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_order(quantity: &str) -> NewOrderRequest {
        NewOrderRequest {
            symbol: "DANGCEM".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: quantity.to_string(),
            limit_price: None,
        }
    }

    #[tokio::test]
    async fn stub_returns_empty_portfolio() {
        let svc = StubPortfolioTradingService::default();
        let snapshot = svc.get_portfolio("user-1").await.unwrap();
        assert_eq!(snapshot.user_id, "user-1");
        assert!(snapshot.balances.is_empty());
    }

    #[tokio::test]
    async fn stub_acknowledges_valid_orders_with_distinct_ids() {
        let svc = StubPortfolioTradingService::default();
        let first = svc.place_order("user-1", sample_order("1")).await.unwrap();
        let second = svc.place_order("user-1", sample_order("2")).await.unwrap();
        assert_ne!(first.ack.order_id, second.ack.order_id);
        assert!(first.trades.is_empty());
    }

    #[tokio::test]
    async fn stub_rejects_invalid_quantities() {
        let svc = StubPortfolioTradingService::default();
        for bad in ["0", "-5", "1.5", "not-a-number"] {
            let err = svc
                .place_order("user-1", sample_order(bad))
                .await
                .unwrap_err();
            assert!(matches!(err, TradingError::InvalidQuantity(_)), "{bad}");
        }
    }
}
