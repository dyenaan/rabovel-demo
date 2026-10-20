use crate::trading::execution::decimal;
use crate::*;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

/// A deterministic placeholder implementation of [`PortfolioTradingApi`].
///
/// Orders are acknowledged but never matched or persisted.
#[derive(Debug, Default)]
pub struct StubPortfolioTradingService {
    next_order_id: AtomicU64,
}

#[async_trait::async_trait]
impl crate::trading::orders::ports::OrderCommands for StubPortfolioTradingService {
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
}

#[async_trait::async_trait]
impl crate::trading::orders::ports::OrderQueries for StubPortfolioTradingService {
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

#[async_trait::async_trait]
impl crate::portfolio::balances::ports::BalanceQueries for StubPortfolioTradingService {
    async fn get_portfolio(&self, user_id: &str) -> Result<PortfolioSnapshot, TradingError> {
        Ok(PortfolioSnapshot {
            user_id: user_id.to_string(),
            balances: vec![],
        })
    }
}

#[async_trait::async_trait]
impl crate::trading::market_queries::ports::MarketQueries for StubPortfolioTradingService {}

#[async_trait::async_trait]
impl crate::trading::repricing::ports::Repricing for StubPortfolioTradingService {}
