//! Capabilities needed by order workflows, independent of HTTP and transport.

use crate::ExecutedTrade;
use crate::NewOrderRequest;
use crate::OrderAck;
use crate::*;
use async_trait::async_trait;

/// Identity established by an inbound adapter. Access policy must still check
/// current ownership, permissions and onboarding before executing a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderActor {
    pub user_id: String,
    pub mfa_verified_at: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderAction {
    Place,
    View,
    Cancel,
}

/// Access gate for each workflow. The adapter owns the existing authorization
/// and throttling policy; its typed rejection survives the application boundary.
#[async_trait]
pub trait OrderAccessPolicy: Send + Sync {
    type Error: Send;

    async fn authorize(&self, actor: &OrderActor, action: OrderAction) -> Result<(), Self::Error>;
}

/// Transport-independent publication attempts. Delivery is currently best-effort:
/// workflows continue after errors because matching has already mutated state.
/// Durable execution and outbox delivery require a separate recovery design.
#[async_trait]
pub trait TradingEventSink: Send + Sync {
    async fn order_accepted(
        &self,
        actor: &OrderActor,
        request: &NewOrderRequest,
        ack: &OrderAck,
    ) -> Result<(), String>;

    async fn trade_executed(&self, trade: ExecutedTrade) -> Result<(), String>;
}

#[async_trait::async_trait]
pub trait OrderCommands: Send + Sync {
    /// Executes the order immediately as far as liquidity allows. The returned
    /// [`OrderExecution::trades`] are for event publication and must not be
    /// returned to the client: they identify counterparties.
    async fn place_order(
        &self,
        user_id: &str,
        order: NewOrderRequest,
    ) -> Result<OrderExecution, TradingError>;
    async fn cancel_order(&self, user_id: &str, order_id: &str) -> Result<(), TradingError>;
}

#[async_trait::async_trait]
pub trait OrderQueries: Send + Sync {
    async fn list_orders(&self, user_id: &str) -> Result<Vec<OrderStatus>, TradingError>;

    async fn get_order(&self, user_id: &str, order_id: &str) -> Result<OrderStatus, TradingError>;
}
