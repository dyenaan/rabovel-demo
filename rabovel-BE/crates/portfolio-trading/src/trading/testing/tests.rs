use crate::BalanceQueries;
use crate::OrderCommands;
use crate::*;

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
