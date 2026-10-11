use super::*;
use crate::{now_secs, InventoryMovement};

/// DANGCEM references at 100 cNGN; the default 50 bps half-spread puts the
/// house at 99.5 bid / 100.5 ask.
struct FixedPrices {
    prices: HashMap<String, ReferencePrice>,
}

#[async_trait::async_trait]
impl ReferencePriceSource for FixedPrices {
    async fn reference_price(
        &self,
        symbol: &str,
        _price_decimals: u8,
    ) -> Result<Option<ReferencePrice>, String> {
        Ok(self.prices.get(symbol).copied())
    }
}

fn service_with(config: EngineTradingConfig, as_of: u64) -> EngineTradingService {
    let prices = FixedPrices {
        prices: HashMap::from([(
            "DANGCEM".to_string(),
            ReferencePrice {
                price_base_units: 100_000_000,
                as_of,
            },
        )]),
    };
    EngineTradingService::new(Arc::new(prices), config)
}

fn service() -> EngineTradingService {
    service_with(EngineTradingConfig::default(), now_secs())
}

fn market(side: OrderSide, quantity: &str) -> NewOrderRequest {
    NewOrderRequest {
        symbol: "dangcem".into(),
        side,
        order_type: OrderType::Market,
        quantity: quantity.into(),
        limit_price: None,
    }
}

fn limit(side: OrderSide, quantity: &str, price: &str) -> NewOrderRequest {
    NewOrderRequest {
        order_type: OrderType::Limit,
        limit_price: Some(price.into()),
        ..market(side, quantity)
    }
}

fn balance<'a>(snapshot: &'a PortfolioSnapshot, asset: &str) -> &'a AssetBalance {
    snapshot
        .balances
        .iter()
        .find(|balance| balance.asset == asset)
        .unwrap_or_else(|| panic!("no {asset} balance in {snapshot:?}"))
}

#[tokio::test]
async fn market_buy_is_filled_instantly_by_the_house() {
    let svc = service();
    let execution = svc
        .place_order("alice", market(OrderSide::Buy, "10"))
        .await
        .unwrap();

    let ack = &execution.ack;
    assert_eq!(ack.state, OrderState::Filled);
    assert_eq!(ack.filled_quantity, "10");
    assert_eq!(ack.average_price.as_deref(), Some("100.5"));
    assert_eq!(ack.fills.len(), 1);
    assert_eq!(ack.fills[0].liquidity, Liquidity::Principal);
    assert_eq!(ack.fills[0].price, "100.5");

    let [trade] = execution.trades.as_slice() else {
        panic!("expected one trade: {:?}", execution.trades);
    };
    assert_eq!(trade.symbol, "DANGCEM");
    assert_eq!(trade.price_base_units, 100_500_000);
    assert_eq!(trade.seller, TradeParty::House);
    assert_eq!(
        trade.buyer,
        TradeParty::Client {
            user_id: "alice".into(),
            order_id: ack.order_id.clone()
        }
    );
    assert_eq!(
        trade.house_inventory,
        Some(InventoryMovement::HouseDelivers {
            from_float: 0,
            minted: 10
        })
    );

    let portfolio = svc.get_portfolio("alice").await.unwrap();
    assert_eq!(balance(&portfolio, "DANGCEM").total, "10");
    assert_eq!(balance(&portfolio, "CNGN").total, "-1005");
}

#[tokio::test]
async fn the_configured_float_is_delivered_before_minting() {
    let svc = service_with(
        EngineTradingConfig {
            initial_float: HashMap::from([("dangcem".to_string(), 4)]),
            ..EngineTradingConfig::default()
        },
        now_secs(),
    );
    let execution = svc
        .place_order("alice", market(OrderSide::Buy, "10"))
        .await
        .unwrap();
    assert_eq!(
        execution.trades[0].house_inventory,
        Some(InventoryMovement::HouseDelivers {
            from_float: 4,
            minted: 6
        })
    );
}

#[tokio::test]
async fn clients_trade_with_each_other_when_the_book_is_better() {
    let svc = service();
    let resting = svc
        .place_order("bob", limit(OrderSide::Sell, "4", "100.2"))
        .await
        .unwrap();
    assert_eq!(resting.ack.state, OrderState::Accepted);
    assert_eq!(resting.ack.resting_quantity, "4");

    let execution = svc
        .place_order("alice", market(OrderSide::Buy, "4"))
        .await
        .unwrap();
    assert_eq!(execution.ack.fills[0].liquidity, Liquidity::Internalized);
    assert_eq!(
        execution.trades[0].seller,
        TradeParty::Client {
            user_id: "bob".into(),
            order_id: resting.ack.order_id.clone()
        }
    );
    assert_eq!(execution.trades[0].house_inventory, None);

    let bob_order = svc.get_order("bob", &resting.ack.order_id).await.unwrap();
    assert_eq!(bob_order.state, OrderState::Filled);
    assert_eq!(bob_order.average_price.as_deref(), Some("100.2"));

    let bob = svc.get_portfolio("bob").await.unwrap();
    assert_eq!(balance(&bob, "DANGCEM").total, "-4");
    assert_eq!(balance(&bob, "CNGN").total, "400.8");
}

#[tokio::test]
async fn resting_orders_lock_balances_until_cancelled() {
    let svc = service();
    let resting = svc
        .place_order("alice", limit(OrderSide::Buy, "5", "99"))
        .await
        .unwrap();

    let portfolio = svc.get_portfolio("alice").await.unwrap();
    let cash = balance(&portfolio, "CNGN");
    assert_eq!(
        (cash.locked.as_str(), cash.available.as_str()),
        ("495", "-495")
    );

    assert_eq!(
        svc.cancel_order("mallory", &resting.ack.order_id).await,
        Err(TradingError::OrderNotFound)
    );
    assert_eq!(
        svc.get_order("mallory", &resting.ack.order_id).await,
        Err(TradingError::OrderNotFound)
    );
    svc.cancel_order("alice", &resting.ack.order_id)
        .await
        .unwrap();

    let order = svc.get_order("alice", &resting.ack.order_id).await.unwrap();
    assert_eq!(order.state, OrderState::Cancelled);
    assert_eq!(order.remaining_quantity, "0");
    // No trades and nothing locked: the snapshot is empty again.
    let portfolio = svc.get_portfolio("alice").await.unwrap();
    assert!(portfolio.balances.is_empty(), "{portfolio:?}");
    assert_eq!(
        svc.cancel_order("alice", &resting.ack.order_id).await,
        Err(TradingError::OrderNotFound)
    );
}

#[tokio::test]
async fn orders_are_listed_newest_first_per_user() {
    let svc = service();
    let first = svc
        .place_order("alice", limit(OrderSide::Buy, "1", "90"))
        .await
        .unwrap();
    let second = svc
        .place_order("alice", market(OrderSide::Buy, "2"))
        .await
        .unwrap();
    svc.place_order("bob", market(OrderSide::Buy, "3"))
        .await
        .unwrap();

    let orders = svc.list_orders("alice").await.unwrap();
    let ids: Vec<&str> = orders.iter().map(|order| order.order_id.as_str()).collect();
    assert_eq!(
        ids,
        vec![second.ack.order_id.as_str(), first.ack.order_id.as_str()]
    );
    assert_eq!(orders[1].limit_price.as_deref(), Some("90"));
    assert_eq!(orders[1].remaining_quantity, "1");
}

#[tokio::test]
async fn stale_reference_prices_stop_house_quoting() {
    let svc = service_with(EngineTradingConfig::default(), now_secs() - 3_600);
    let execution = svc
        .place_order("alice", market(OrderSide::Buy, "5"))
        .await
        .unwrap();

    assert_eq!(execution.ack.state, OrderState::Cancelled);
    assert_eq!(execution.ack.cancelled_quantity, "5");
    assert_eq!(
        execution.ack.unfilled_reason.as_deref(),
        Some("no_reference_price")
    );
}

#[tokio::test]
async fn invalid_orders_are_rejected() {
    let svc = service();
    let unknown = NewOrderRequest {
        symbol: "NOPE".into(),
        ..market(OrderSide::Buy, "1")
    };
    assert_eq!(
        svc.place_order("alice", unknown).await,
        Err(TradingError::UnknownSymbol("NOPE".into()))
    );
    assert!(matches!(
        svc.place_order("alice", market(OrderSide::Buy, "1.5"))
            .await,
        Err(TradingError::InvalidQuantity(_))
    ));
    assert!(matches!(
        svc.place_order("alice", limit(OrderSide::Buy, "1", "100.0000001"))
            .await,
        Err(TradingError::InvalidPrice(_))
    ));
    let limit_without_price = NewOrderRequest {
        order_type: OrderType::Limit,
        ..market(OrderSide::Buy, "1")
    };
    assert!(matches!(
        svc.place_order("alice", limit_without_price).await,
        Err(TradingError::InvalidOrder(_))
    ));
    assert!(matches!(
        svc.place_order("alice", market(OrderSide::Buy, "10001"))
            .await,
        Err(TradingError::OrderTooLarge(_))
    ));
}

#[tokio::test]
async fn book_merges_resting_orders_with_the_house_quote_and_trades_are_taped() {
    let svc = service();
    svc.place_order("bob", limit(OrderSide::Buy, "4", "99"))
        .await
        .unwrap();
    svc.place_order("alice", market(OrderSide::Buy, "3"))
        .await
        .unwrap();

    let book = svc.order_book("dangcem", 10).await.unwrap();
    assert_eq!(book.symbol, "DANGCEM");
    // House bid 99.5 ranks above bob's 99 bid; the house ask is 100.5.
    assert_eq!(book.bids[0].price_base_units, 99_500_000);
    assert_eq!(
        book.bids[1],
        BookLevel {
            price_base_units: 99_000_000,
            quantity: 4
        }
    );
    assert_eq!(book.asks[0].price_base_units, 100_500_000);
    assert!(book.asks[0].quantity > 0);

    let trades = svc.public_trades("DANGCEM", 10).await.unwrap();
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 3);
    assert_eq!(trades[0].price_base_units, 100_500_000);
    assert_eq!(trades[0].aggressor, OrderSide::Buy);
    assert!(svc.public_trades("GTCO", 10).await.unwrap().is_empty());
}

/// A reference price the test can move.
struct MovablePrice(std::sync::Mutex<u64>);

#[async_trait::async_trait]
impl ReferencePriceSource for MovablePrice {
    async fn reference_price(
        &self,
        _symbol: &str,
        _price_decimals: u8,
    ) -> Result<Option<ReferencePrice>, String> {
        Ok(Some(ReferencePrice {
            price_base_units: *self.0.lock().unwrap(),
            as_of: now_secs(),
        }))
    }
}

#[tokio::test]
async fn resting_limits_fill_when_the_market_moves_through_them() {
    let prices = Arc::new(MovablePrice(std::sync::Mutex::new(100_000_000)));
    let svc = EngineTradingService::new(prices.clone(), EngineTradingConfig::default());
    let ack = svc
        .place_order("bob", limit(OrderSide::Buy, "2", "99"))
        .await
        .unwrap()
        .ack;
    assert_eq!(ack.state, OrderState::Accepted);
    assert!(svc.reprice_resting().await.unwrap().is_empty());

    // The market falls: the house ask (98 + 0.5%) is now below bob's 99 bid.
    *prices.0.lock().unwrap() = 98_000_000;
    let trades = svc.reprice_resting().await.unwrap();
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 2);
    let order = svc.get_order("bob", &ack.order_id).await.unwrap();
    assert_eq!(order.state, OrderState::Filled);
    assert_eq!(svc.public_trades("DANGCEM", 5).await.unwrap().len(), 1);
}
