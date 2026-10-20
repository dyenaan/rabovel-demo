use super::*;
use crate::ExecutedTrade;
use crate::OrderSide;
use crate::OrderState;
use crate::OrderType;
use crate::PublicTrade;
use crate::TradingError;
use matching_engine::Fill;
use matching_engine::MatchingEngine;
use matching_engine::OrderId;
use matching_engine::Participant;
use matching_engine::Price;
use matching_engine::Qty;
use matching_engine::Side;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;

pub(super) struct OrderRecord {
    pub(super) user_id: String,
    pub(super) symbol: String,
    pub(super) side: OrderSide,
    pub(super) order_type: OrderType,
    pub(super) quantity: Qty,
    pub(super) limit_price: Option<Price>,
    pub(super) filled: Qty,
    pub(super) notional: u128,
    pub(super) state: OrderState,
    pub(super) created_at: u64,
    pub(super) updated_at: u64,
}
#[derive(Default)]
pub(super) struct TradingState {
    pub(super) engine: MatchingEngine,
    pub(super) orders: HashMap<OrderId, OrderRecord>,
    pub(super) user_orders: HashMap<String, Vec<OrderId>>,
    /// user → asset → net base units (tokens are whole units).
    pub(super) balances: HashMap<String, BTreeMap<String, i128>>,
    /// symbol → recent trades, newest at the back.
    pub(super) public_trades: HashMap<String, VecDeque<PublicTrade>>,
}
impl TradingState {
    /// Applies fills to order records and balances, returning trade records.
    pub(super) fn apply_fills(
        &mut self,
        fills: &[Fill],
        now: u64,
        price_decimals: u8,
    ) -> Vec<ExecutedTrade> {
        fills
            .iter()
            .map(|fill| {
                let notional = u128::from(fill.quantity) * u128::from(fill.price);
                for (participant, side) in [(&fill.buyer, Side::Buy), (&fill.seller, Side::Sell)] {
                    let Participant::Client { account, order_id } = participant else {
                        continue;
                    };
                    if let Some(record) = self.orders.get_mut(order_id) {
                        record.filled += fill.quantity;
                        record.notional += notional;
                        record.state = if record.filled == record.quantity {
                            OrderState::Filled
                        } else {
                            OrderState::PartiallyFilled
                        };
                        record.updated_at = now;
                    }
                    let balances = self.balances.entry(account.clone()).or_default();
                    let signed_quantity = i128::from(fill.quantity);
                    let signed_notional = notional as i128;
                    let (asset_delta, cash_delta) = match side {
                        Side::Buy => (signed_quantity, -signed_notional),
                        Side::Sell => (-signed_quantity, signed_notional),
                    };
                    *balances.entry(fill.symbol.clone()).or_default() += asset_delta;
                    *balances.entry(String::new()).or_default() += cash_delta;
                }
                let tape = self.public_trades.entry(fill.symbol.clone()).or_default();
                if tape.len() == MAX_PUBLIC_TRADES {
                    tape.pop_front();
                }
                tape.push_back(PublicTrade {
                    trade_id: format!("trd_{}", fill.trade_id.0),
                    symbol: fill.symbol.clone(),
                    price_base_units: fill.price,
                    quantity: fill.quantity,
                    aggressor: from_engine_side(fill.aggressor),
                    executed_at: now,
                });
                ExecutedTrade {
                    trade_id: format!("trd_{}", fill.trade_id.0),
                    sequence: fill.sequence,
                    symbol: fill.symbol.clone(),
                    quantity: fill.quantity,
                    price_base_units: fill.price,
                    price_decimals,
                    buyer: trade_party(&fill.buyer),
                    seller: trade_party(&fill.seller),
                    liquidity: liquidity(fill.source),
                    aggressor: from_engine_side(fill.aggressor),
                    house_inventory: fill.house_inventory,
                    executed_at: now,
                }
            })
            .collect()
    }

    pub(super) fn mark_cancelled(&mut self, id: OrderId, now: u64) {
        if let Some(record) = self.orders.get_mut(&id) {
            record.state = OrderState::Cancelled;
            record.updated_at = now;
        }
    }

    pub(super) fn owned_order(
        &self,
        user_id: &str,
        order_id: &str,
    ) -> Result<OrderId, TradingError> {
        parse_order_id(order_id)
            .filter(|id| {
                self.orders
                    .get(id)
                    .is_some_and(|record| record.user_id == user_id)
            })
            .ok_or(TradingError::OrderNotFound)
    }

    pub(super) fn remaining(&self, user_id: &str, id: OrderId) -> Qty {
        self.engine
            .open_orders(user_id)
            .into_iter()
            .find(|open| open.order_id == id)
            .map_or(0, |open| open.remaining_quantity)
    }
}
