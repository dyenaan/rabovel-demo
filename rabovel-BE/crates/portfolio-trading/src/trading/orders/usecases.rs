//! Order orchestration shared by HTTP commands and background repricing.

use crate::NewOrderRequest;
use crate::OrderAccessPolicy;
use crate::OrderAck;
use crate::OrderAction;
use crate::OrderActor;
use crate::OrderStatus;
use crate::PortfolioTradingApi;
use crate::TradingError;
use crate::TradingEventSink;
use std::sync::Arc;

/// Keeps policy rejections distinct from execution failures so inbound adapters
/// can preserve their existing status, error code and message mappings.
#[derive(Debug, thiserror::Error)]
pub enum OrderWorkflowError<E> {
    #[error("{0}")]
    Access(E),
    #[error(transparent)]
    Trading(#[from] TradingError),
}

pub struct OrderUseCases<E> {
    commands: Arc<dyn crate::OrderCommands>,
    queries: Arc<dyn crate::OrderQueries>,
    repricing: Arc<dyn crate::Repricing>,
    access: Arc<dyn OrderAccessPolicy<Error = E>>,
    events: Arc<dyn TradingEventSink>,
}

impl<E: Send> OrderUseCases<E> {
    pub fn new(
        engine: Arc<dyn PortfolioTradingApi>,
        access: Arc<dyn OrderAccessPolicy<Error = E>>,
        events: Arc<dyn TradingEventSink>,
    ) -> Self {
        Self {
            commands: engine.clone(),
            queries: engine.clone(),
            repricing: engine,
            access,
            events,
        }
    }

    pub async fn place_order(
        &self,
        actor: &OrderActor,
        request: NewOrderRequest,
    ) -> Result<OrderAck, OrderWorkflowError<E>> {
        self.access
            .authorize(actor, OrderAction::Place)
            .await
            .map_err(OrderWorkflowError::Access)?;
        let execution = self
            .commands
            .place_order(&actor.user_id, request.clone())
            .await?;
        let ack = execution.ack;

        // Matching is non-idempotent today. A publication failure must not turn
        // successful execution into an error that invites the caller to retry.
        let _ = self.events.order_accepted(actor, &request, &ack).await;
        for trade in execution.trades {
            let _ = self.events.trade_executed(trade).await;
        }
        // Execution details include counterparties; only the ack is public.
        Ok(ack)
    }

    pub async fn get_order(
        &self,
        actor: &OrderActor,
        order_id: &str,
    ) -> Result<OrderStatus, OrderWorkflowError<E>> {
        self.access
            .authorize(actor, OrderAction::View)
            .await
            .map_err(OrderWorkflowError::Access)?;
        Ok(self.queries.get_order(&actor.user_id, order_id).await?)
    }

    pub async fn list_orders(
        &self,
        actor: &OrderActor,
    ) -> Result<Vec<OrderStatus>, OrderWorkflowError<E>> {
        self.access
            .authorize(actor, OrderAction::View)
            .await
            .map_err(OrderWorkflowError::Access)?;
        Ok(self.queries.list_orders(&actor.user_id).await?)
    }

    pub async fn cancel_order(
        &self,
        actor: &OrderActor,
        order_id: &str,
    ) -> Result<(), OrderWorkflowError<E>> {
        self.access
            .authorize(actor, OrderAction::Cancel)
            .await
            .map_err(OrderWorkflowError::Access)?;
        Ok(self.commands.cancel_order(&actor.user_id, order_id).await?)
    }

    /// System workflow: uses the same engine and publication path as client
    /// commands, without inventing an acceptance event for resting orders.
    pub async fn reprice_resting(&self) -> Result<(), TradingError> {
        for trade in self.repricing.reprice_resting().await? {
            let _ = self.events.trade_executed(trade).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
