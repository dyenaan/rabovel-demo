use crate::trading::execution::decimal::format_fixed;
use crate::OrderStatus;
use crate::TradingError;
use conversions::*;
use matching_engine::EngineConfig;
use matching_engine::MatchingEngine;
use matching_engine::OrderId;
use matching_engine::Qty;
use state::OrderRecord;
use state::TradingState;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

/// Recent public trades kept per symbol for the tape and price history.
const MAX_PUBLIC_TRADES: usize = 1_000;
/// A reference price in payment-asset base units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferencePrice {
    pub price_base_units: u64,
    /// Unix seconds the price was observed.
    pub as_of: u64,
}
/// Port for tradable instruments and their reference prices.
#[async_trait::async_trait]
pub trait ReferencePriceSource: Send + Sync {
    /// `Ok(None)` when `symbol` is not a tradable instrument; `Err` when the
    /// source cannot answer.
    async fn reference_price(
        &self,
        symbol: &str,
        price_decimals: u8,
    ) -> Result<Option<ReferencePrice>, String>;
}
#[derive(Debug, Clone)]
pub struct EngineTradingConfig {
    pub engine: EngineConfig,
    /// Decimals of the payment asset prices are quoted in.
    pub price_decimals: u8,
    /// Balance name of the payment asset.
    pub payment_asset: String,
    /// Older reference prices are ignored: the house stops quoting and only
    /// client-to-client matching remains.
    pub max_price_age_secs: u64,
    /// Treasury float pre-funded per symbol at startup.
    pub initial_float: HashMap<String, Qty>,
}
impl Default for EngineTradingConfig {
    fn default() -> Self {
        Self {
            engine: EngineConfig::default(),
            price_decimals: 6,
            payment_asset: "CNGN".into(),
            max_price_age_secs: 60,
            initial_float: HashMap::new(),
        }
    }
}
pub struct EngineTradingService {
    prices: Arc<dyn ReferencePriceSource>,
    config: EngineTradingConfig,
    state: Mutex<TradingState>,
}
impl EngineTradingService {
    pub fn new(prices: Arc<dyn ReferencePriceSource>, config: EngineTradingConfig) -> Self {
        let mut engine = MatchingEngine::new(config.engine.clone());
        for (symbol, quantity) in &config.initial_float {
            engine.fund_float(&normalize_symbol(symbol), *quantity);
        }
        Self {
            prices,
            config,
            state: Mutex::new(TradingState {
                engine,
                ..TradingState::default()
            }),
        }
    }

    fn lock(&self) -> Result<MutexGuard<'_, TradingState>, TradingError> {
        self.state
            .lock()
            .map_err(|_| TradingError::Unavailable("trading state is unavailable".into()))
    }

    fn order_status(&self, id: OrderId, record: &OrderRecord, remaining: Qty) -> OrderStatus {
        let decimals = self.config.price_decimals;
        OrderStatus {
            order_id: format_order_id(id),
            symbol: record.symbol.clone(),
            side: record.side,
            order_type: record.order_type,
            limit_price: record
                .limit_price
                .map(|price| format_fixed(price.into(), decimals)),
            quantity: record.quantity.to_string(),
            filled_quantity: record.filled.to_string(),
            remaining_quantity: remaining.to_string(),
            average_price: average_price(record.notional, record.filled, decimals),
            state: record.state,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}
mod conversions;
mod orders;
mod queries;
mod repricing;
mod state;
#[cfg(test)]
mod tests;
