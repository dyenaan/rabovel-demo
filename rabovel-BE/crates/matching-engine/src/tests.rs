use super::*;

const SYMBOL: &str = "RABO-NG-TELCO";
/// 100 cNGN at 6 decimals; with the default 50 bps half-spread the house
/// quotes 99.5 bid / 100.5 ask.
const REF: Price = 100_000_000;
const HOUSE_BID: Price = 99_500_000;
const HOUSE_ASK: Price = 100_500_000;

fn engine() -> MatchingEngine {
    MatchingEngine::new(EngineConfig::default())
}

fn engine_with_limits(limits: HouseLimits) -> MatchingEngine {
    MatchingEngine::new(EngineConfig {
        default_limits: limits,
        symbol_limits: HashMap::new(),
    })
}

fn order(account: &str, side: Side, kind: OrderKind, quantity: Qty) -> NewOrder {
    NewOrder {
        account: account.to_string(),
        symbol: SYMBOL.to_string(),
        side,
        kind,
        quantity,
    }
}

fn limit(price: Price) -> OrderKind {
    OrderKind::Limit { price }
}

fn client(account: &str, order_id: OrderId) -> Participant {
    Participant::Client {
        account: account.to_string(),
        order_id,
    }
}

#[test]
fn market_buy_on_empty_book_is_filled_instantly_by_the_house_at_its_ask() {
    let mut engine = engine();
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 10), Some(REF))
        .unwrap();

    assert_eq!(report.filled_quantity, 10);
    assert_eq!(report.cancelled_quantity, 0);
    assert_eq!(report.fills.len(), 1);
    let fill = &report.fills[0];
    assert_eq!(fill.price, HOUSE_ASK);
    assert_eq!(fill.seller, Participant::House);
    assert_eq!(fill.buyer, client("alice", report.order_id));
    assert_eq!(fill.source, LiquiditySource::Principal);
    assert_eq!(
        engine.house_position(SYMBOL),
        HousePosition {
            net_quantity: -10,
            net_cash: 10 * i128::from(HOUSE_ASK),
            float: 0,
            minted_unbacked: 10,
        }
    );
}

#[test]
fn client_orders_priced_better_than_the_house_are_internalized_first() {
    let mut engine = engine();
    let maker = engine
        .submit(order("bob", Side::Sell, limit(100_200_000), 4), Some(REF))
        .unwrap();
    assert_eq!(maker.resting_quantity, 4);

    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 10), Some(REF))
        .unwrap();

    assert_eq!(report.filled_quantity, 10);
    let [book_fill, house_fill] = report.fills.as_slice() else {
        panic!("expected two fills: {:?}", report.fills);
    };
    assert_eq!(book_fill.source, LiquiditySource::Internalized);
    assert_eq!(book_fill.house_inventory, None);
    assert_eq!(book_fill.seller, client("bob", maker.order_id));
    assert_eq!((book_fill.quantity, book_fill.price), (4, 100_200_000));
    assert_eq!(house_fill.source, LiquiditySource::Principal);
    assert_eq!((house_fill.quantity, house_fill.price), (6, HOUSE_ASK));
    assert!(engine.open_orders("bob").is_empty());
}

#[test]
fn equal_prices_prefer_the_book_so_the_house_stays_flat() {
    let mut engine = engine();
    engine
        .submit(order("bob", Side::Sell, limit(HOUSE_ASK), 5), Some(REF))
        .unwrap();
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 5), Some(REF))
        .unwrap();

    assert_eq!(report.fills[0].source, LiquiditySource::Internalized);
    assert_eq!(engine.house_position(SYMBOL), HousePosition::default());
}

#[test]
fn the_house_fills_when_resting_client_prices_are_worse() {
    let mut engine = engine();
    engine
        .submit(order("bob", Side::Sell, limit(101_000_000), 5), Some(REF))
        .unwrap();
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 5), Some(REF))
        .unwrap();

    assert_eq!(report.fills.len(), 1);
    assert_eq!(report.fills[0].seller, Participant::House);
    assert_eq!(engine.open_orders("bob")[0].remaining_quantity, 5);
}

#[test]
fn market_orders_beyond_house_risk_capacity_are_partially_filled() {
    let mut engine = engine_with_limits(HouseLimits {
        max_short: 10,
        ..HouseLimits::default()
    });
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 15), Some(REF))
        .unwrap();

    assert_eq!(report.filled_quantity, 10);
    assert_eq!(report.cancelled_quantity, 5);
    assert_eq!(report.unfilled_reason, Some(UnfilledReason::HouseRiskLimit));

    // Client selling back restores capacity on the other side.
    engine
        .submit(order("bob", Side::Sell, OrderKind::Market, 3), Some(REF))
        .unwrap();
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 5), Some(REF))
        .unwrap();
    assert_eq!(report.filled_quantity, 3);
}

#[test]
fn market_orders_without_book_or_reference_price_are_cancelled() {
    let mut engine = engine();
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 5), None)
        .unwrap();

    assert!(report.fills.is_empty());
    assert_eq!(report.cancelled_quantity, 5);
    assert_eq!(
        report.unfilled_reason,
        Some(UnfilledReason::NoReferencePrice)
    );
}

#[test]
fn clients_still_match_each_other_without_a_reference_price() {
    let mut engine = engine();
    engine
        .submit(order("bob", Side::Buy, limit(98_000_000), 5), None)
        .unwrap();
    let report = engine
        .submit(order("alice", Side::Sell, OrderKind::Market, 5), None)
        .unwrap();

    assert_eq!(report.filled_quantity, 5);
    assert_eq!(report.fills[0].price, 98_000_000);
}

#[test]
fn crossing_limit_orders_execute_at_the_resting_price() {
    let mut engine = engine();
    engine
        .submit(order("bob", Side::Buy, limit(99_800_000), 5), Some(REF))
        .unwrap();
    // Alice will sell down to 99.0; bob's 99.8 bid beats the house's 99.5 bid.
    let report = engine
        .submit(order("alice", Side::Sell, limit(99_000_000), 5), Some(REF))
        .unwrap();

    assert_eq!(report.fills.len(), 1);
    assert_eq!(report.fills[0].price, 99_800_000);
    assert_eq!(report.fills[0].aggressor, Side::Sell);
    assert_eq!(report.fills[0].buyer, client("bob", OrderId(1)));
}

#[test]
fn equal_priced_resting_orders_fill_in_time_priority() {
    let mut engine = engine();
    let first = engine
        .submit(order("bob", Side::Sell, limit(100_000_000), 3), Some(REF))
        .unwrap();
    let second = engine
        .submit(order("carol", Side::Sell, limit(100_000_000), 3), Some(REF))
        .unwrap();
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 4), Some(REF))
        .unwrap();

    assert_eq!(report.fills[0].seller, client("bob", first.order_id));
    assert_eq!(report.fills[0].quantity, 3);
    assert_eq!(report.fills[1].seller, client("carol", second.order_id));
    assert_eq!(report.fills[1].quantity, 1);
    assert_eq!(engine.open_orders("carol")[0].remaining_quantity, 2);
}

#[test]
fn self_trade_prevention_cancels_the_resting_order() {
    let mut engine = engine();
    let own = engine
        .submit(order("alice", Side::Sell, limit(100_000_000), 5), Some(REF))
        .unwrap();
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 5), Some(REF))
        .unwrap();

    assert_eq!(report.self_trade_cancellations, vec![own.order_id]);
    assert!(report
        .fills
        .iter()
        .all(|fill| fill.seller == Participant::House));
    assert!(engine.open_orders("alice").is_empty());
}

#[test]
fn reprice_fills_resting_limits_the_new_house_quote_satisfies() {
    let mut engine = engine();
    let resting = engine
        .submit(order("alice", Side::Buy, limit(99_000_000), 5), Some(REF))
        .unwrap();
    assert_eq!(resting.resting_quantity, 5);

    // Reference drops to 98: house ask becomes 98.49, inside alice's 99 limit.
    let fills = engine.reprice(SYMBOL, 98_000_000);

    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].price, 98_490_000);
    assert_eq!(fills[0].buyer, client("alice", resting.order_id));
    assert_eq!(fills[0].seller, Participant::House);
    assert!(engine.open_orders("alice").is_empty());
}

#[test]
fn cancel_only_succeeds_for_the_owner() {
    let mut engine = engine();
    let resting = engine
        .submit(order("alice", Side::Buy, limit(90_000_000), 5), Some(REF))
        .unwrap();

    assert_eq!(
        engine.cancel("mallory", resting.order_id),
        Err(EngineError::OrderNotFound)
    );
    let cancelled = engine.cancel("alice", resting.order_id).unwrap();
    assert_eq!(cancelled.remaining_quantity, 5);
    assert_eq!(
        engine.cancel("alice", resting.order_id),
        Err(EngineError::OrderNotFound)
    );
    assert!(engine.depth(SYMBOL, 5).bids.is_empty());
}

#[test]
fn closing_the_hedge_window_nets_principal_flow_per_symbol() {
    let mut engine = engine();
    engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 10), Some(REF))
        .unwrap();
    engine
        .submit(order("bob", Side::Sell, OrderKind::Market, 4), Some(REF))
        .unwrap();

    let instructions = engine.close_hedge_window();
    assert_eq!(
        instructions,
        vec![HedgeInstruction {
            symbol: SYMBOL.to_string(),
            hedge_side: Some(Side::Buy),
            hedge_quantity: 6,
            house_bought: 4,
            house_sold: 10,
            buy_notional: 4 * u128::from(HOUSE_BID),
            sell_notional: 10 * u128::from(HOUSE_ASK),
            minted_in_window: 10,
            // Bob's 4 sold back to the house retired 4 of the 10 minted.
            minted_unbacked: 6,
            principal_fills: 2,
        }]
    );
    assert!(engine.close_hedge_window().is_empty());
    // The position carries over; only the window resets.
    assert_eq!(engine.house_position(SYMBOL).net_quantity, -6);
}

#[test]
fn invalid_orders_are_rejected_before_matching() {
    let mut engine = engine();
    assert_eq!(
        engine.submit(order("alice", Side::Buy, OrderKind::Market, 0), Some(REF)),
        Err(EngineError::InvalidQuantity)
    );
    assert_eq!(
        engine.submit(order("alice", Side::Buy, limit(0), 1), Some(REF)),
        Err(EngineError::InvalidPrice)
    );
    assert_eq!(
        engine.submit(
            order("alice", Side::Buy, OrderKind::Market, 10_001),
            Some(REF)
        ),
        Err(EngineError::OrderTooLarge {
            requested: 10_001,
            max: 10_000
        })
    );
}

#[test]
fn trade_sequences_are_gap_free_across_submit_and_reprice() {
    let mut engine = engine();
    engine
        .submit(order("alice", Side::Buy, limit(99_000_000), 2), Some(REF))
        .unwrap();
    let a = engine
        .submit(order("bob", Side::Buy, OrderKind::Market, 1), Some(REF))
        .unwrap();
    let b = engine.reprice(SYMBOL, 98_000_000);
    let sequences: Vec<u64> = a.fills.iter().chain(&b).map(|fill| fill.sequence).collect();
    assert_eq!(sequences, vec![1, 2]);
}

#[test]
fn house_sales_deliver_from_float_first_and_mint_only_the_shortfall() {
    let mut engine = engine();
    engine.fund_float(SYMBOL, 4);
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 10), Some(REF))
        .unwrap();

    assert_eq!(report.filled_quantity, 10);
    assert_eq!(
        report.fills[0].house_inventory,
        Some(InventoryMovement::HouseDelivers {
            from_float: 4,
            minted: 6
        })
    );
    let position = engine.house_position(SYMBOL);
    assert_eq!((position.float, position.minted_unbacked), (0, 6));
}

#[test]
fn a_zero_mint_limit_restricts_delivery_to_the_float() {
    let mut engine = engine_with_limits(HouseLimits {
        max_minted_unbacked: 0,
        ..HouseLimits::default()
    });
    engine.fund_float(SYMBOL, 5);
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 8), Some(REF))
        .unwrap();

    assert_eq!(report.filled_quantity, 5);
    assert_eq!(report.cancelled_quantity, 3);
    assert_eq!(report.unfilled_reason, Some(UnfilledReason::InventoryLimit));
    assert_eq!(engine.house_position(SYMBOL).minted_unbacked, 0);
}

#[test]
fn the_unbacked_mint_limit_caps_delivery_once_the_float_is_empty() {
    let mut engine = engine_with_limits(HouseLimits {
        max_minted_unbacked: 3,
        ..HouseLimits::default()
    });
    engine.fund_float(SYMBOL, 2);
    let report = engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 10), Some(REF))
        .unwrap();

    assert_eq!(report.filled_quantity, 5);
    assert_eq!(report.unfilled_reason, Some(UnfilledReason::InventoryLimit));
}

#[test]
fn house_purchases_retire_unbacked_mints_before_refilling_the_float() {
    let mut engine = engine();
    engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 10), Some(REF))
        .unwrap();

    let first = engine
        .submit(order("bob", Side::Sell, OrderKind::Market, 3), Some(REF))
        .unwrap();
    assert_eq!(
        first.fills[0].house_inventory,
        Some(InventoryMovement::HouseReceives {
            to_float: 0,
            retired: 3
        })
    );
    let second = engine
        .submit(order("bob", Side::Sell, OrderKind::Market, 9), Some(REF))
        .unwrap();
    assert_eq!(
        second.fills[0].house_inventory,
        Some(InventoryMovement::HouseReceives {
            to_float: 2,
            retired: 7
        })
    );
    let position = engine.house_position(SYMBOL);
    assert_eq!((position.float, position.minted_unbacked), (2, 0));
}

#[test]
fn settled_hedge_buys_back_mints_first_then_refill_the_float() {
    let mut engine = engine();
    engine
        .submit(order("alice", Side::Buy, OrderKind::Market, 10), Some(REF))
        .unwrap();

    let settlement = engine.settle_hedge(SYMBOL, Side::Buy, 12).unwrap();
    assert_eq!(
        settlement,
        HedgeSettlement {
            backed: 10,
            float_change: 2
        }
    );
    let position = engine.house_position(SYMBOL);
    assert_eq!((position.float, position.minted_unbacked), (2, 0));

    assert_eq!(
        engine.settle_hedge(SYMBOL, Side::Sell, 5),
        Err(EngineError::InsufficientFloat)
    );
    assert_eq!(
        engine
            .settle_hedge(SYMBOL, Side::Sell, 2)
            .unwrap()
            .float_change,
        2
    );
    assert_eq!(engine.house_position(SYMBOL).float, 0);
}
