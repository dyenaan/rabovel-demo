use crate::app::errors::ApplicationError;
use crate::app::support::state_unavailable;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::identity::wallets::queries::usecases::investor_solana_wallet;
use crate::features::market_data::charts::calculations as market_views;
use crate::features::market_data::prices::usecases::reference_price;
use crate::features::portfolio::valuation::dto::HoldingResponse;
use crate::features::portfolio::valuation::dto::PortfolioSummaryResponse;
use crate::features::settlement::direct_purchases::catalog::usecases::investor_catalog_for_user;
use identity::AuthError;
use portfolio_trading::OrderSide;
use std::collections::HashMap;

/// One engine position valued at the current reference price.
pub(crate) struct PositionView {
    pub(crate) asset_id: String,
    pub(crate) name: String,
    pub(crate) ticker: String,
    pub(crate) quantity: i128,
    pub(crate) cost_basis: i128,
    pub(crate) price: i128,
}

impl PositionView {
    pub(crate) fn market_value(&self) -> i128 {
        self.quantity * self.price
    }
}

pub(crate) struct PortfolioView {
    pub(crate) positions: Vec<PositionView>,
    pub(crate) cash: i128,
}

/// The caller's engine positions, average cost from their own fills, and the
/// linked wallet's on-chain cNGN balance as cash (zero when unavailable).
/// Engine balances are matched trades that are not yet settled on-chain.
pub(crate) async fn portfolio_view(
    state: &impl Ports,
    user_id: &str,
) -> Result<PortfolioView, ApplicationError> {
    let decimals = state.price_decimals();
    let snapshot = state
        .balance_queries()
        .get_portfolio(user_id)
        .await
        .map_err(ApplicationError::from)?;
    let mut orders = state
        .order_queries()
        .list_orders(user_id)
        .await
        .map_err(ApplicationError::from)?;
    orders.sort_by_key(|order| order.created_at);
    // symbol -> (quantity, cost) under the average-cost method.
    let mut lots: HashMap<String, (i128, i128)> = HashMap::new();
    for order in &orders {
        let filled = market_views::parse_units(&order.filled_quantity, 0).unwrap_or(0);
        let average = order
            .average_price
            .as_deref()
            .and_then(|price| market_views::parse_units(price, decimals))
            .unwrap_or(0);
        if filled == 0 {
            continue;
        }
        let (quantity, cost) = lots.entry(order.symbol.clone()).or_default();
        match order.side {
            OrderSide::Buy => {
                *quantity += filled;
                *cost += filled * average;
            }
            OrderSide::Sell => {
                if *quantity > 0 {
                    *cost -= *cost * filled.min(*quantity) / *quantity;
                }
                *quantity -= filled;
            }
        }
    }

    let mut positions = Vec::new();
    for balance in snapshot.balances {
        if balance.asset.eq_ignore_ascii_case("CNGN") {
            continue;
        }
        let quantity = market_views::parse_units(&balance.total, 0).unwrap_or(0);
        if quantity <= 0 {
            continue;
        }
        let (asset_id, name) = match state.instrument_provider().find(&balance.asset).await {
            Ok(Some(instrument)) => (instrument.asset_id, instrument.name),
            _ => (balance.asset.to_ascii_lowercase(), balance.asset.clone()),
        };
        positions.push(PositionView {
            cost_basis: lots
                .get(&balance.asset)
                .map_or(0, |(_, cost)| (*cost).max(0)),
            price: reference_price(state, &balance.asset)?.into(),
            asset_id,
            name,
            ticker: balance.asset,
            quantity,
        });
    }

    // Wallet purchases and engine trades are separate positions today: the
    // engine does not settle its fills into these token accounts. Include both
    // quantities, but count the wallet's cash only once. Propagate RPC failures
    // rather than displaying a partial portfolio as a verified zero balance.
    let has_wallet = match investor_solana_wallet(state, user_id).await {
        Ok(_) => true,
        Err(error)
            if matches!(
                error.error,
                AuthError::MissingWallet | AuthError::ForbiddenAction
            ) =>
        {
            false
        }
        Err(error) => return Err(error),
    };
    let cash = if has_wallet {
        let catalog = investor_catalog_for_user(state, user_id).await?;
        for asset in catalog.assets {
            let quantity = asset
                .investor_balance
                .parse::<i128>()
                .map_err(|_| ApplicationError::from(state_unavailable()))?;
            if quantity <= 0 {
                continue;
            }
            let price = reference_price(state, &asset.ticker)?;
            merge_wallet_position(
                &mut positions,
                asset.asset_id,
                asset.name,
                asset.ticker,
                quantity,
                price.into(),
            );
        }
        catalog
            .cngn
            .balance_base_units
            .as_deref()
            .zip(catalog.cngn.decimals)
            .and_then(|(units, source_decimals)| {
                let units = units.parse::<i128>().ok()?;
                let source_scale = 10_i128.checked_pow(source_decimals.into())?;
                let target_scale = 10_i128.checked_pow(decimals.into())?;
                units.checked_mul(target_scale)?.checked_div(source_scale)
            })
            .ok_or_else(|| ApplicationError::from(state_unavailable()))?
    } else {
        0
    };
    Ok(PortfolioView { positions, cash })
}

/// Preserve the previous wallet-view valuation baseline until purchase-lot
/// accounting exists. It is an estimate of cost, not a recorded acquisition price.
pub(crate) fn merge_wallet_position(
    positions: &mut Vec<PositionView>,
    asset_id: String,
    name: String,
    ticker: String,
    quantity: i128,
    price: i128,
) {
    let baseline_cost = quantity * price;
    if let Some(position) = positions
        .iter_mut()
        .find(|position| position.asset_id == asset_id)
    {
        position.quantity += quantity;
        position.cost_basis += baseline_cost;
    } else {
        positions.push(PositionView {
            asset_id,
            name,
            ticker,
            quantity,
            cost_basis: baseline_cost,
            price,
        });
    }
}

pub(crate) async fn portfolio_summary(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<PortfolioSummaryResponse, ApplicationError> {
    let view = portfolio_view(state, &user.user_id).await?;
    let decimals = state.price_decimals();
    let invested: i128 = view.positions.iter().map(PositionView::market_value).sum();
    let cost: i128 = view
        .positions
        .iter()
        .map(|position| position.cost_basis)
        .sum();
    let fixed = |value: i128| market_views::fixed2(value, decimals);
    Ok(PortfolioSummaryResponse {
        total_value: fixed(invested + view.cash),
        total_cost_basis: fixed(cost),
        total_unrealized_pnl: fixed(invested - cost),
        total_unrealized_pnl_percent: market_views::percent(invested - cost, cost),
        cash_balance: fixed(view.cash),
        invested_value: fixed(invested),
        ytd_income: fixed(0),
    })
}

pub(crate) async fn portfolio_holdings(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<Vec<HoldingResponse>, ApplicationError> {
    let view = portfolio_view(state, &user.user_id).await?;
    let decimals = state.price_decimals();
    let invested: i128 = view.positions.iter().map(PositionView::market_value).sum();
    let fixed = |value: i128| market_views::fixed2(value, decimals);
    Ok(view
        .positions
        .iter()
        .map(|position| {
            let value = position.market_value();
            HoldingResponse {
                asset_id: position.asset_id.clone(),
                asset_name: position.name.clone(),
                symbol: position.ticker.clone(),
                quantity: position.quantity.to_string(),
                average_cost: fixed(position.cost_basis / position.quantity),
                market_value: fixed(value),
                cost_basis: fixed(position.cost_basis),
                unrealized_pnl: fixed(value - position.cost_basis),
                unrealized_pnl_percent: market_views::percent(
                    value - position.cost_basis,
                    position.cost_basis,
                ),
                allocation_percent: market_views::percent(value, invested),
            }
        })
        .collect())
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::assets::listings::ports::PublishedAssets
    + crate::features::portfolio::balances::ports::Balances
    + crate::features::catalog::instruments::ports::CatalogSource
    + crate::features::issuer::assets::issuance::inventory::ports::Inventory
    + crate::features::trading::orders::ports::Orders
    + crate::features::payments::cngn::ports::PaymentProvider
    + crate::features::issuer::assets::issuance::inventory::ports::Positions
    + crate::features::market_data::prices::ports::PriceDecimals
    + crate::features::market_data::prices::ports::ReferencePrices
    + crate::features::issuer::assets::issuance::setup::ports::SetupLookup
    + crate::features::identity::wallets::queries::ports::WalletLookup
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::assets::listings::ports::PublishedAssets
            + crate::features::portfolio::balances::ports::Balances
            + crate::features::catalog::instruments::ports::CatalogSource
            + crate::features::issuer::assets::issuance::inventory::ports::Inventory
            + crate::features::trading::orders::ports::Orders
            + crate::features::payments::cngn::ports::PaymentProvider
            + crate::features::issuer::assets::issuance::inventory::ports::Positions
            + crate::features::market_data::prices::ports::PriceDecimals
            + crate::features::market_data::prices::ports::ReferencePrices
            + crate::features::issuer::assets::issuance::setup::ports::SetupLookup
            + crate::features::identity::wallets::queries::ports::WalletLookup
            + ?Sized,
    > Ports for T
{
}
