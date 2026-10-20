use super::*;
use crate::now_secs;
use crate::trading::execution::decimal::format_fixed;
use crate::trading::execution::decimal::parse_fixed;
use crate::NewOrderRequest;
use crate::OrderAck;
use crate::OrderExecution;
use crate::OrderFill;
use crate::OrderState;
use crate::OrderType;
use crate::TradingError;
use matching_engine::NewOrder;
use matching_engine::OrderKind;

#[async_trait::async_trait]
impl crate::trading::orders::ports::OrderCommands for EngineTradingService {
    async fn place_order(
        &self,
        user_id: &str,
        order: NewOrderRequest,
    ) -> Result<OrderExecution, TradingError> {
        let decimals = self.config.price_decimals;
        let symbol = normalize_symbol(&order.symbol);
        if symbol.is_empty() {
            return Err(TradingError::UnknownSymbol(order.symbol));
        }
        let quantity = parse_fixed(&order.quantity, 0)
            .filter(|quantity| *quantity > 0)
            .ok_or_else(|| TradingError::InvalidQuantity(order.quantity.clone()))?;
        let (kind, limit_price) = match (order.order_type, order.limit_price.as_deref()) {
            (OrderType::Market, None) => (OrderKind::Market, None),
            (OrderType::Market, Some(_)) => {
                return Err(TradingError::InvalidOrder(
                    "market orders do not take a limit price".into(),
                ))
            }
            (OrderType::Limit, Some(raw)) => {
                let price = parse_fixed(raw, decimals)
                    .filter(|price| *price > 0)
                    .ok_or_else(|| TradingError::InvalidPrice(raw.to_string()))?;
                (OrderKind::Limit { price }, Some(price))
            }
            (OrderType::Limit, None) => {
                return Err(TradingError::InvalidOrder(
                    "limit orders require a limit price".into(),
                ))
            }
        };
        let reference = self
            .prices
            .reference_price(&symbol, decimals)
            .await
            .map_err(TradingError::Unavailable)?
            .ok_or_else(|| TradingError::UnknownSymbol(symbol.clone()))?;

        let now = now_secs();
        let fresh = (now.saturating_sub(reference.as_of) <= self.config.max_price_age_secs)
            .then_some(reference.price_base_units);

        let mut state = self.lock()?;
        let mut trades = Vec::new();
        if let Some(price) = fresh {
            // Fill resting limits the current quote already satisfies, so the
            // new order never trades ahead of them.
            let fills = state.engine.reprice(&symbol, price);
            trades.extend(state.apply_fills(&fills, now, decimals));
        }

        let report = state
            .engine
            .submit(
                NewOrder {
                    account: user_id.to_string(),
                    symbol: symbol.clone(),
                    side: to_engine_side(order.side),
                    kind,
                    quantity,
                },
                fresh,
            )
            .map_err(map_engine_error)?;

        state.orders.insert(
            report.order_id,
            OrderRecord {
                user_id: user_id.to_string(),
                symbol,
                side: order.side,
                order_type: order.order_type,
                quantity,
                limit_price,
                filled: 0,
                notional: 0,
                state: OrderState::Accepted,
                created_at: now,
                updated_at: now,
            },
        );
        state
            .user_orders
            .entry(user_id.to_string())
            .or_default()
            .push(report.order_id);
        for cancelled in &report.self_trade_cancellations {
            state.mark_cancelled(*cancelled, now);
        }
        trades.extend(state.apply_fills(&report.fills, now, decimals));

        let record = state
            .orders
            .get_mut(&report.order_id)
            .expect("order was just inserted");
        record.state = if report.resting_quantity > 0 {
            if report.filled_quantity > 0 {
                OrderState::PartiallyFilled
            } else {
                OrderState::Accepted
            }
        } else if report.cancelled_quantity > 0 {
            OrderState::Cancelled
        } else {
            OrderState::Filled
        };

        let ack = OrderAck {
            order_id: format_order_id(report.order_id),
            accepted_at: now,
            state: record.state,
            filled_quantity: report.filled_quantity.to_string(),
            resting_quantity: report.resting_quantity.to_string(),
            cancelled_quantity: report.cancelled_quantity.to_string(),
            average_price: average_price(record.notional, record.filled, decimals),
            fills: report
                .fills
                .iter()
                .map(|fill| OrderFill {
                    trade_id: format!("trd_{}", fill.trade_id.0),
                    quantity: fill.quantity.to_string(),
                    price: format_fixed(fill.price.into(), decimals),
                    liquidity: liquidity(fill.source),
                })
                .collect(),
            unfilled_reason: report
                .unfilled_reason
                .map(|reason| unfilled_reason(reason).into()),
        };
        Ok(OrderExecution { ack, trades })
    }
    async fn cancel_order(&self, user_id: &str, order_id: &str) -> Result<(), TradingError> {
        let mut state = self.lock()?;
        let id = state.owned_order(user_id, order_id)?;
        state.engine.cancel(user_id, id).map_err(map_engine_error)?;
        state.mark_cancelled(id, now_secs());
        Ok(())
    }
}
