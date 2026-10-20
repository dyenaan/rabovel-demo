use super::*;
use crate::EngineTradingConfig;
use crate::EngineTradingService;
use crate::ExecutedTrade;
use crate::Liquidity;
use crate::OrderExecution;
use crate::OrderQueries;
use crate::OrderSide;
use crate::OrderType;
use crate::PortfolioSnapshot;
use crate::ReferencePrice;
use crate::ReferencePriceSource;
use crate::StubPortfolioTradingService;
use crate::TradeParty;
use async_trait::async_trait;
use std::sync::Mutex;

#[derive(Default)]
struct Probe {
    trace: Mutex<Vec<String>>,
    deny_access: bool,
    fail_engine: bool,
    fail_publication: bool,
}

impl Probe {
    fn record(&self, message: impl Into<String>) {
        self.trace.lock().unwrap().push(message.into());
    }

    fn trace(&self) -> Vec<String> {
        self.trace.lock().unwrap().clone()
    }

    fn workflows(self: &Arc<Self>) -> OrderUseCases<&'static str> {
        OrderUseCases::new(self.clone(), self.clone(), self.clone())
    }
}

#[async_trait]
impl OrderAccessPolicy for Probe {
    type Error = &'static str;

    async fn authorize(&self, actor: &OrderActor, action: OrderAction) -> Result<(), Self::Error> {
        assert_eq!(actor.mfa_verified_at, Some(42));
        self.record(format!("access:{action:?}"));
        if self.deny_access {
            Err("denied")
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl TradingEventSink for Probe {
    async fn order_accepted(
        &self,
        actor: &OrderActor,
        request: &NewOrderRequest,
        ack: &OrderAck,
    ) -> Result<(), String> {
        assert_eq!(ack.resting_quantity, request.quantity);
        self.record(format!(
            "accepted:{}:{}:{}",
            actor.user_id, request.symbol, request.quantity
        ));
        if self.fail_publication {
            Err("broker unavailable".into())
        } else {
            Ok(())
        }
    }

    async fn trade_executed(&self, trade: ExecutedTrade) -> Result<(), String> {
        self.record(format!("trade:{}", trade.trade_id));
        if self.fail_publication {
            Err("broker unavailable".into())
        } else {
            Ok(())
        }
    }
}

#[async_trait::async_trait]
impl crate::trading::orders::ports::OrderCommands for Probe {
    async fn place_order(
        &self,
        user_id: &str,
        request: NewOrderRequest,
    ) -> Result<OrderExecution, TradingError> {
        self.record(format!("engine:place:{user_id}"));
        if self.fail_engine {
            return Err(TradingError::Unavailable("engine unavailable".into()));
        }
        let mut execution = StubPortfolioTradingService::default()
            .place_order(user_id, request)
            .await?;
        // The first trade can belong to an earlier resting order. Both must
        // survive orchestration, in engine order, without leaking into the ack.
        execution.trades = vec![trade(1), trade(2)];
        Ok(execution)
    }
    async fn cancel_order(&self, user_id: &str, order_id: &str) -> Result<(), TradingError> {
        self.record(format!("engine:cancel:{user_id}:{order_id}"));
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::trading::orders::ports::OrderQueries for Probe {
    async fn get_order(&self, user_id: &str, order_id: &str) -> Result<OrderStatus, TradingError> {
        self.record(format!("engine:get:{user_id}:{order_id}"));
        Err(TradingError::OrderNotFound)
    }
    async fn list_orders(&self, user_id: &str) -> Result<Vec<OrderStatus>, TradingError> {
        self.record(format!("engine:list:{user_id}"));
        Ok(vec![])
    }
}

#[async_trait::async_trait]
impl crate::portfolio::balances::ports::BalanceQueries for Probe {
    async fn get_portfolio(&self, _: &str) -> Result<PortfolioSnapshot, TradingError> {
        panic!("order workflows must not load portfolio projections")
    }
}

#[async_trait::async_trait]
impl crate::trading::market_queries::ports::MarketQueries for Probe {}

#[async_trait::async_trait]
impl crate::trading::repricing::ports::Repricing for Probe {
    async fn reprice_resting(&self) -> Result<Vec<ExecutedTrade>, TradingError> {
        self.record("engine:reprice");
        if self.fail_engine {
            Err(TradingError::Unavailable("engine unavailable".into()))
        } else {
            Ok(vec![trade(1), trade(2)])
        }
    }
}

fn actor(user_id: &str) -> OrderActor {
    OrderActor {
        user_id: user_id.into(),
        mfa_verified_at: Some(42),
    }
}

fn order() -> NewOrderRequest {
    NewOrderRequest {
        symbol: "dangcem".into(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: "3".into(),
        limit_price: None,
    }
}

fn trade(sequence: u64) -> ExecutedTrade {
    ExecutedTrade {
        trade_id: format!("trd_{sequence}"),
        sequence,
        symbol: "DANGCEM".into(),
        quantity: 1,
        price_base_units: 100_000_000,
        price_decimals: 6,
        buyer: TradeParty::Client {
            user_id: "other-user".into(),
            order_id: "resting-order".into(),
        },
        seller: TradeParty::House,
        liquidity: Liquidity::Principal,
        aggressor: OrderSide::Buy,
        house_inventory: None,
        executed_at: 1,
    }
}

#[tokio::test]
async fn denied_commands_and_queries_never_execute_or_publish() {
    let probe = Arc::new(Probe {
        deny_access: true,
        ..Probe::default()
    });
    let workflows = probe.workflows();
    let actor = actor("alice");
    assert!(matches!(
        workflows.place_order(&actor, order()).await,
        Err(OrderWorkflowError::Access("denied"))
    ));
    assert!(matches!(
        workflows.get_order(&actor, "ord_1").await,
        Err(OrderWorkflowError::Access("denied"))
    ));
    assert!(matches!(
        workflows.list_orders(&actor).await,
        Err(OrderWorkflowError::Access("denied"))
    ));
    assert!(matches!(
        workflows.cancel_order(&actor, "ord_1").await,
        Err(OrderWorkflowError::Access("denied"))
    ));
    assert_eq!(
        probe.trace(),
        [
            "access:Place",
            "access:View",
            "access:View",
            "access:Cancel"
        ]
    );
}

#[tokio::test]
async fn acceptance_preserves_request_and_precedes_all_execution_trades() {
    let probe = Arc::new(Probe::default());
    let ack = probe
        .workflows()
        .place_order(&actor("alice"), order())
        .await
        .unwrap();
    assert_eq!(ack.resting_quantity, "3");
    assert_eq!(
        probe.trace(),
        [
            "access:Place",
            "engine:place:alice",
            "accepted:alice:dangcem:3",
            "trade:trd_1",
            "trade:trd_2"
        ]
    );
}

#[tokio::test]
async fn publication_failures_still_return_ack_without_reexecution_or_skipped_trades() {
    let probe = Arc::new(Probe {
        fail_publication: true,
        ..Probe::default()
    });
    assert!(probe
        .workflows()
        .place_order(&actor("alice"), order())
        .await
        .is_ok());
    assert_eq!(
        probe.trace(),
        [
            "access:Place",
            "engine:place:alice",
            "accepted:alice:dangcem:3",
            "trade:trd_1",
            "trade:trd_2"
        ]
    );
}

#[tokio::test]
async fn engine_failure_retains_typed_error_without_publishing() {
    let probe = Arc::new(Probe {
        fail_engine: true,
        ..Probe::default()
    });
    assert!(matches!(
        probe.workflows().place_order(&actor("alice"), order()).await,
        Err(OrderWorkflowError::Trading(TradingError::Unavailable(message)))
            if message == "engine unavailable"
    ));
    assert_eq!(probe.trace(), ["access:Place", "engine:place:alice"]);
}

#[tokio::test]
async fn queries_and_cancellation_keep_actor_and_distinct_access_actions() {
    let probe = Arc::new(Probe::default());
    let workflows = probe.workflows();
    let actor = actor("alice");
    assert!(workflows.get_order(&actor, "ord_1").await.is_err());
    assert!(workflows.list_orders(&actor).await.unwrap().is_empty());
    workflows.cancel_order(&actor, "ord_1").await.unwrap();
    assert_eq!(
        probe.trace(),
        [
            "access:View",
            "engine:get:alice:ord_1",
            "access:View",
            "engine:list:alice",
            "access:Cancel",
            "engine:cancel:alice:ord_1"
        ]
    );
}

#[tokio::test]
async fn repricing_attempts_every_trade_without_access_or_acceptance_even_when_broker_fails() {
    let probe = Arc::new(Probe {
        deny_access: true,
        fail_publication: true,
        ..Probe::default()
    });
    probe.workflows().reprice_resting().await.unwrap();
    assert_eq!(
        probe.trace(),
        ["engine:reprice", "trade:trd_1", "trade:trd_2"]
    );
}

#[tokio::test]
async fn repricing_failure_retains_engine_error_without_publishing() {
    let probe = Arc::new(Probe {
        fail_engine: true,
        ..Probe::default()
    });
    assert_eq!(
        probe.workflows().reprice_resting().await,
        Err(TradingError::Unavailable("engine unavailable".into()))
    );
    assert_eq!(probe.trace(), ["engine:reprice"]);
}

struct FixedPrices;

#[async_trait]
impl ReferencePriceSource for FixedPrices {
    async fn reference_price(&self, _: &str, _: u8) -> Result<Option<ReferencePrice>, String> {
        Ok(Some(ReferencePrice {
            price_base_units: 100_000_000,
            as_of: crate::now_secs(),
        }))
    }
}

#[tokio::test]
async fn workflows_cannot_read_or_cancel_another_actors_real_engine_order() {
    let probe = Arc::new(Probe::default());
    let engine = Arc::new(EngineTradingService::new(
        Arc::new(FixedPrices),
        EngineTradingConfig::default(),
    ));
    let workflows = OrderUseCases::new(engine.clone(), probe.clone(), probe);
    let alice = actor("alice");
    let bob = actor("bob");
    let ack = workflows
        .place_order(
            &alice,
            NewOrderRequest {
                order_type: OrderType::Limit,
                limit_price: Some("1".into()),
                ..order()
            },
        )
        .await
        .unwrap();
    assert!(matches!(
        workflows.get_order(&bob, &ack.order_id).await,
        Err(OrderWorkflowError::Trading(TradingError::OrderNotFound))
    ));
    assert!(matches!(
        workflows.cancel_order(&bob, &ack.order_id).await,
        Err(OrderWorkflowError::Trading(TradingError::OrderNotFound))
    ));
    assert!(workflows.list_orders(&bob).await.unwrap().is_empty());
    assert_eq!(
        engine
            .get_order("alice", &ack.order_id)
            .await
            .unwrap()
            .remaining_quantity,
        "3"
    );
    workflows.cancel_order(&alice, &ack.order_id).await.unwrap();
    assert_eq!(
        engine
            .get_order("alice", &ack.order_id)
            .await
            .unwrap()
            .state,
        crate::OrderState::Cancelled
    );
}
