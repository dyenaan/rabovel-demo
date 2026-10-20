//! Kafka/no-op transport bridge. This adapter preserves existing envelopes and
//! producer names; reliable delivery belongs to the later outbox slice.

use async_trait::async_trait;
use events::schema::trading::HouseInventory;
use events::schema::trading::OrderAccepted;
use events::schema::trading::OrderSide as EventOrderSide;
use events::schema::trading::TradeExecuted;
use events::schema::trading::TradeLiquidity;
use events::schema::trading::TradeParty as EventTradeParty;
use events::EventProducer;
use infrastructure::messaging;
use portfolio_trading::ExecutedTrade;
use portfolio_trading::InventoryMovement;
use portfolio_trading::Liquidity;
use portfolio_trading::NewOrderRequest;
use portfolio_trading::OrderAck;
use portfolio_trading::OrderActor;
use portfolio_trading::OrderSide;
use portfolio_trading::TradeParty;
use portfolio_trading::TradingEventSink;
use std::sync::Arc;

pub(in crate::features::trading) struct MessagingTradingEvents {
    pub producer: Arc<dyn EventProducer>,
}

#[async_trait]
impl TradingEventSink for MessagingTradingEvents {
    async fn order_accepted(
        &self,
        actor: &OrderActor,
        request: &NewOrderRequest,
        ack: &OrderAck,
    ) -> Result<(), String> {
        messaging::publish(
            self.producer.as_ref(),
            OrderAccepted {
                order_id: ack.order_id.clone(),
                user_id: actor.user_id.clone(),
                symbol: request.symbol.clone(),
                side: to_event_order_side(request.side),
                quantity: request.quantity.clone(),
                accepted_at: ack.accepted_at,
            },
            "order-gateway",
        )
        .await
        .map_err(|error| error.to_string())
    }

    async fn trade_executed(&self, trade: ExecutedTrade) -> Result<(), String> {
        messaging::publish(
            self.producer.as_ref(),
            to_trade_executed(trade),
            "matching-engine",
        )
        .await
        .map_err(|error| error.to_string())
    }
}

fn to_event_order_side(side: OrderSide) -> EventOrderSide {
    match side {
        OrderSide::Buy => EventOrderSide::Buy,
        OrderSide::Sell => EventOrderSide::Sell,
    }
}

fn to_trade_executed(trade: ExecutedTrade) -> TradeExecuted {
    let party = |party: TradeParty| match party {
        TradeParty::Client { user_id, order_id } => EventTradeParty::Client { user_id, order_id },
        TradeParty::House => EventTradeParty::House,
    };
    TradeExecuted {
        trade_id: trade.trade_id,
        sequence: trade.sequence,
        symbol: trade.symbol,
        quantity: trade.quantity.to_string(),
        price_base_units: trade.price_base_units.to_string(),
        price_decimals: trade.price_decimals,
        buyer: party(trade.buyer),
        seller: party(trade.seller),
        liquidity: match trade.liquidity {
            Liquidity::Internalized => TradeLiquidity::Internalized,
            Liquidity::Principal => TradeLiquidity::Principal,
        },
        aggressor: to_event_order_side(trade.aggressor),
        house_inventory: trade.house_inventory.map(|movement| match movement {
            InventoryMovement::HouseDelivers { from_float, minted } => {
                HouseInventory::HouseDelivers { from_float, minted }
            }
            InventoryMovement::HouseReceives { to_float, retired } => {
                HouseInventory::HouseReceives { to_float, retired }
            }
        }),
        executed_at: trade.executed_at,
    }
}
