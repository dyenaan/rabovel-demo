//! Per-symbol client limit order book with price-time priority.

use std::collections::{BTreeMap, VecDeque};

use crate::{OrderId, Price, Qty, Side};

#[derive(Debug, Clone)]
pub(crate) struct RestingOrder {
    pub id: OrderId,
    pub account: String,
    pub original: Qty,
    pub remaining: Qty,
}

#[derive(Debug, Default)]
pub(crate) struct OrderBook {
    bids: BTreeMap<Price, VecDeque<RestingOrder>>,
    asks: BTreeMap<Price, VecDeque<RestingOrder>>,
}

impl OrderBook {
    fn levels(&self, side: Side) -> &BTreeMap<Price, VecDeque<RestingOrder>> {
        match side {
            Side::Buy => &self.bids,
            Side::Sell => &self.asks,
        }
    }

    fn levels_mut(&mut self, side: Side) -> &mut BTreeMap<Price, VecDeque<RestingOrder>> {
        match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        }
    }

    /// Best price resting on `side`: highest bid or lowest ask.
    pub fn best_price(&self, side: Side) -> Option<Price> {
        match side {
            Side::Buy => self.bids.keys().next_back().copied(),
            Side::Sell => self.asks.keys().next().copied(),
        }
    }

    /// The oldest order at the best price on `side`.
    pub fn front_mut(&mut self, side: Side) -> Option<(Price, &mut RestingOrder)> {
        let price = self.best_price(side)?;
        let order = self.levels_mut(side).get_mut(&price)?.front_mut()?;
        Some((price, order))
    }

    /// Removes the oldest order at the best price on `side`.
    pub fn pop_front(&mut self, side: Side) -> Option<RestingOrder> {
        let price = self.best_price(side)?;
        let levels = self.levels_mut(side);
        let queue = levels.get_mut(&price)?;
        let order = queue.pop_front();
        if queue.is_empty() {
            levels.remove(&price);
        }
        order
    }

    pub fn insert(&mut self, side: Side, price: Price, order: RestingOrder) {
        self.levels_mut(side)
            .entry(price)
            .or_default()
            .push_back(order);
    }

    pub fn remove(&mut self, side: Side, price: Price, id: OrderId) -> Option<RestingOrder> {
        let levels = self.levels_mut(side);
        let queue = levels.get_mut(&price)?;
        let index = queue.iter().position(|order| order.id == id)?;
        let order = queue.remove(index);
        if queue.is_empty() {
            levels.remove(&price);
        }
        order
    }

    pub fn get(&self, side: Side, price: Price, id: OrderId) -> Option<&RestingOrder> {
        self.levels(side)
            .get(&price)?
            .iter()
            .find(|order| order.id == id)
    }

    /// Aggregated `(price, quantity)` levels, best first.
    pub fn depth(&self, side: Side, max_levels: usize) -> Vec<(Price, Qty)> {
        let aggregate = |(price, queue): (&Price, &VecDeque<RestingOrder>)| -> (Price, Qty) {
            (*price, queue.iter().map(|order| order.remaining).sum())
        };
        match side {
            Side::Buy => self
                .bids
                .iter()
                .rev()
                .take(max_levels)
                .map(aggregate)
                .collect(),
            Side::Sell => self.asks.iter().take(max_levels).map(aggregate).collect(),
        }
    }
}
