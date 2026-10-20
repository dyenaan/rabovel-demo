use crate::features::portfolio::valuation::usecases::merge_wallet_position;
use crate::features::portfolio::valuation::usecases::PositionView;

#[test]
fn portfolio_keeps_wallet_purchases_and_engine_positions_without_duplicate_rows() {
    let mut positions = vec![PositionView {
        asset_id: "ast_demo".into(),
        name: "Demo".into(),
        ticker: "DEMO".into(),
        quantity: 2,
        cost_basis: 18_000_000,
        price: 10_000_000,
    }];
    merge_wallet_position(
        &mut positions,
        "ast_demo".into(),
        "Demo".into(),
        "DEMO".into(),
        3,
        10_000_000,
    );
    merge_wallet_position(
        &mut positions,
        "ast_other".into(),
        "Other".into(),
        "OTHER".into(),
        4,
        5_000_000,
    );
    assert_eq!(positions.len(), 2);
    assert_eq!(positions[0].quantity, 5);
    assert_eq!(positions[0].cost_basis, 48_000_000);
    assert_eq!(
        positions
            .iter()
            .map(PositionView::market_value)
            .sum::<i128>(),
        70_000_000
    );
}
