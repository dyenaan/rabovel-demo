//! Contract-only crate for the portfolio trading API.
//!
//! This crate defines the [`PortfolioTradingApi`] trait and its associated DTOs,
//! along with a deterministic stub implementation ([`StubPortfolioTradingService`]).
//! Real trading/matching logic is out of scope here and will be provided by a
//! separate matching-engine-backed implementation later.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// The portfolio trading API contract.
///
/// Implementations are responsible for exposing a user's portfolio and for
/// accepting/cancelling/listing orders. Amounts are represented as `String`
/// rather than floats to avoid float-precision foot-guns in a financial
/// context.
#[async_trait::async_trait]
pub trait PortfolioTradingApi: Send + Sync {
    async fn get_portfolio(&self, user_id: &str) -> Result<PortfolioSnapshot, TradingError>;
    async fn place_order(
        &self,
        user_id: &str,
        order: NewOrderRequest,
    ) -> Result<OrderAck, TradingError>;
    async fn cancel_order(&self, user_id: &str, order_id: &str) -> Result<(), TradingError>;
    async fn list_orders(&self, user_id: &str) -> Result<Vec<OrderStatus>, TradingError>;
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

/// A request to place a new order.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NewOrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: String,
    pub limit_price: Option<String>,
}

/// Acknowledgement returned after an order is accepted.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OrderAck {
    pub order_id: String,
    pub accepted_at: u64,
}

/// The lifecycle state of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderState {
    Accepted,
    PartiallyFilled,
    Filled,
    Cancelled,
}

/// The current status of an order.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OrderStatus {
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: String,
    pub filled_quantity: String,
    pub state: OrderState,
}

/// Errors that can occur while interacting with the portfolio trading API.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TradingError {
    #[error("order not found")]
    OrderNotFound,
    #[error("invalid quantity: {0}")]
    InvalidQuantity(String),
    #[error("service unavailable: {0}")]
    Unavailable(String),
}

/// A deterministic placeholder implementation of [`PortfolioTradingApi`].
///
/// This stub does not perform any real trading or matching: orders are never
/// actually matched, so there is nothing to persist or track beyond a
/// monotonically increasing order-id counter. A colleague will replace this
/// with a real matching-engine-backed implementation; the BFF route wiring
/// around this trait does not need to change when that happens.
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
    ) -> Result<OrderAck, TradingError> {
        let quantity: f64 = order
            .quantity
            .parse()
            .map_err(|_| TradingError::InvalidQuantity(order.quantity.clone()))?;
        if quantity.is_nan() || quantity <= 0.0 {
            return Err(TradingError::InvalidQuantity(order.quantity.clone()));
        }

        let n = self.next_order_id.fetch_add(1, Ordering::SeqCst);
        let accepted_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(OrderAck {
            order_id: format!("ord_{n}"),
            accepted_at,
        })
    }

    async fn cancel_order(&self, _user_id: &str, _order_id: &str) -> Result<(), TradingError> {
        // Nothing is actually tracked yet, so there is nothing to cancel.
        Ok(())
    }

    async fn list_orders(&self, _user_id: &str) -> Result<Vec<OrderStatus>, TradingError> {
        // No orders are persisted by this stub.
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_order(quantity: &str) -> NewOrderRequest {
        NewOrderRequest {
            symbol: "BTC-USD".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: quantity.to_string(),
            limit_price: None,
        }
    }

    #[tokio::test]
    async fn get_portfolio_returns_empty_balances() {
        let svc = StubPortfolioTradingService::default();
        let snapshot = svc.get_portfolio("user-1").await.unwrap();
        assert_eq!(snapshot.user_id, "user-1");
        assert!(snapshot.balances.is_empty());
    }

    #[tokio::test]
    async fn place_order_with_valid_quantity_returns_ack() {
        let svc = StubPortfolioTradingService::default();
        let ack = svc
            .place_order("user-1", sample_order("1.5"))
            .await
            .unwrap();
        assert!(!ack.order_id.is_empty());
    }

    #[tokio::test]
    async fn place_order_rejects_zero_quantity() {
        let svc = StubPortfolioTradingService::default();
        let err = svc
            .place_order("user-1", sample_order("0"))
            .await
            .unwrap_err();
        assert!(matches!(err, TradingError::InvalidQuantity(_)));
    }

    #[tokio::test]
    async fn place_order_rejects_negative_quantity() {
        let svc = StubPortfolioTradingService::default();
        let err = svc
            .place_order("user-1", sample_order("-5"))
            .await
            .unwrap_err();
        assert!(matches!(err, TradingError::InvalidQuantity(_)));
    }

    #[tokio::test]
    async fn place_order_rejects_unparseable_quantity() {
        let svc = StubPortfolioTradingService::default();
        let err = svc
            .place_order("user-1", sample_order("not-a-number"))
            .await
            .unwrap_err();
        assert!(matches!(err, TradingError::InvalidQuantity(_)));
    }

    #[tokio::test]
    async fn successive_place_order_calls_return_different_order_ids() {
        let svc = StubPortfolioTradingService::default();
        let ack1 = svc.place_order("user-1", sample_order("1")).await.unwrap();
        let ack2 = svc.place_order("user-1", sample_order("2")).await.unwrap();
        assert_ne!(ack1.order_id, ack2.order_id);
    }
}
