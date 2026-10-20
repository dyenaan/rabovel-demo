use super::*;
use crate::GatewayStateBuilder;
use async_trait::async_trait;
use events::EventProducer;
use infrastructure::messaging::MessagingError;
use portfolio_trading::EngineTradingConfig;
use portfolio_trading::EngineTradingService;
use portfolio_trading::NewOrderRequest;
use portfolio_trading::OrderCommands;
use portfolio_trading::OrderQueries;
use portfolio_trading::OrderSide;
use portfolio_trading::OrderState;
use portfolio_trading::OrderType;
use portfolio_trading::ReferencePrice;
use portfolio_trading::ReferencePriceSource;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::mpsc;

struct MovingPrices(AtomicU64);

#[async_trait]
impl ReferencePriceSource for MovingPrices {
    async fn reference_price(&self, _: &str, _: u8) -> Result<Option<ReferencePrice>, String> {
        Ok(Some(ReferencePrice {
            price_base_units: self.0.load(Ordering::SeqCst),
            as_of: crate::app::support::unix_now(),
        }))
    }
}

struct ChannelProducer(mpsc::UnboundedSender<(String, Vec<u8>)>);

#[async_trait]
impl EventProducer for ChannelProducer {
    async fn publish_raw(
        &self,
        topic: &str,
        _: &str,
        payload: Vec<u8>,
    ) -> Result<(), MessagingError> {
        self.0.send((topic.into(), payload)).unwrap();
        Ok(())
    }
}

#[tokio::test]
async fn immediate_tick_reprices_the_injected_engine_and_publishes_only_the_fill() {
    let prices = Arc::new(MovingPrices(AtomicU64::new(100_000_000)));
    let engine = Arc::new(EngineTradingService::new(
        prices.clone(),
        EngineTradingConfig::default(),
    ));
    let ack = engine
        .place_order(
            "alice",
            NewOrderRequest {
                symbol: "DANGCEM".into(),
                side: OrderSide::Buy,
                order_type: OrderType::Limit,
                quantity: "3".into(),
                limit_price: Some("1".into()),
            },
        )
        .await
        .unwrap()
        .ack;
    let (sender, mut receiver) = mpsc::unbounded_channel();
    let state = GatewayStateBuilder::default()
        .portfolio_trading(engine.clone())
        .event_producer(Arc::new(ChannelProducer(sender)))
        .build();
    prices.0.store(500_000, Ordering::SeqCst);

    // Default composition has not started a ticker or mutated the resting order.
    assert_eq!(
        engine
            .get_order("alice", &ack.order_id)
            .await
            .unwrap()
            .state,
        OrderState::Accepted
    );
    spawn_market_ticker(state.order_use_cases.clone());
    let (topic, payload) = tokio::time::timeout(Duration::from_secs(2), receiver.recv())
        .await
        .expect("the first tick should run immediately")
        .unwrap();
    assert_eq!(topic, events::topics::TRADING_TRADE_EXECUTED_V1);
    let event: serde_json::Value = serde_json::from_slice(&payload).unwrap();
    assert_eq!(event["producer"], "matching-engine");
    assert_eq!(event["payload"]["buyer"]["order_id"], ack.order_id);
    assert_eq!(event["payload"]["quantity"], "3");
    assert!(
        receiver.try_recv().is_err(),
        "repricing must not emit OrderAccepted"
    );

    // The feature facade, builder injection and legacy portfolio/market state
    // all see the same mutable engine rather than independently built copies.
    assert_eq!(
        state
            .order_queries
            .get_order("alice", &ack.order_id)
            .await
            .unwrap()
            .state,
        OrderState::Filled
    );
    assert_eq!(
        engine
            .get_order("alice", &ack.order_id)
            .await
            .unwrap()
            .filled_quantity,
        "3"
    );
}
