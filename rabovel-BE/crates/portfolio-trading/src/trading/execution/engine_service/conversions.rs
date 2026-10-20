use crate::trading::execution::decimal::format_fixed;
use crate::Liquidity;
use crate::OrderSide;
use crate::TradeParty;
use crate::TradingError;
use matching_engine::EngineError;
use matching_engine::LiquiditySource;
use matching_engine::OrderId;
use matching_engine::Participant;
use matching_engine::Qty;
use matching_engine::Side;
use matching_engine::UnfilledReason;

pub(super) fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_ascii_uppercase()
}
pub(super) fn format_order_id(id: OrderId) -> String {
    format!("ord_{}", id.0)
}
pub(super) fn parse_order_id(order_id: &str) -> Option<OrderId> {
    order_id.strip_prefix("ord_")?.parse().ok().map(OrderId)
}
pub(super) fn average_price(notional: u128, filled: Qty, decimals: u8) -> Option<String> {
    (filled > 0).then(|| format_fixed(notional / u128::from(filled), decimals))
}
pub(super) fn to_engine_side(side: OrderSide) -> Side {
    match side {
        OrderSide::Buy => Side::Buy,
        OrderSide::Sell => Side::Sell,
    }
}
pub(super) fn from_engine_side(side: Side) -> OrderSide {
    match side {
        Side::Buy => OrderSide::Buy,
        Side::Sell => OrderSide::Sell,
    }
}
pub(super) fn liquidity(source: LiquiditySource) -> Liquidity {
    match source {
        LiquiditySource::Internalized => Liquidity::Internalized,
        LiquiditySource::Principal => Liquidity::Principal,
    }
}
pub(super) fn trade_party(participant: &Participant) -> TradeParty {
    match participant {
        Participant::Client { account, order_id } => TradeParty::Client {
            user_id: account.clone(),
            order_id: format_order_id(*order_id),
        },
        Participant::House => TradeParty::House,
    }
}
pub(super) fn unfilled_reason(reason: UnfilledReason) -> &'static str {
    match reason {
        UnfilledReason::NoReferencePrice => "no_reference_price",
        UnfilledReason::NoHouseQuote => "no_house_quote",
        UnfilledReason::HouseRiskLimit => "house_risk_limit",
        UnfilledReason::InventoryLimit => "inventory_limit",
    }
}
pub(super) fn map_engine_error(error: EngineError) -> TradingError {
    match error {
        EngineError::EmptySymbol => TradingError::UnknownSymbol(String::new()),
        EngineError::InvalidQuantity => TradingError::InvalidQuantity(error.to_string()),
        EngineError::InvalidPrice => TradingError::InvalidPrice(error.to_string()),
        EngineError::OrderTooLarge { .. } => TradingError::OrderTooLarge(error.to_string()),
        EngineError::OrderNotFound => TradingError::OrderNotFound,
        EngineError::InsufficientFloat => TradingError::Unavailable(error.to_string()),
    }
}
