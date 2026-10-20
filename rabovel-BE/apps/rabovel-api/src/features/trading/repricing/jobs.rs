//! Process lifecycle stays in API composition; repricing behavior is reusable.

use crate::features::trading::TradingUseCases;
use std::sync::Arc;

pub(crate) fn spawn_market_ticker(orders: Arc<TradingUseCases>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            crate::features::market_data::prices::PRICE_TICK_SECS,
        ));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if let Err(error) = orders.reprice_resting().await {
                eprintln!("market tick failed: {error}");
            }
        }
    });
}

#[cfg(test)]
mod tests;
