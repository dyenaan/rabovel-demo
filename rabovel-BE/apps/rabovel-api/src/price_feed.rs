use async_trait::async_trait;
use portfolio_trading::{ReferencePrice, ReferencePriceSource};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::instrument_provider::InstrumentProvider;

#[derive(Debug, Clone)]
pub struct MarketPrice {
    pub price_base_units: u64,
    pub source: String,
    pub as_of: u64,
}

pub trait PriceFeed: Send + Sync {
    /// The price `ticker` had (or has) at unix second `at`.
    fn price_at(&self, ticker: &str, payment_decimals: u8, at: u64) -> Result<MarketPrice, String>;

    fn latest(&self, ticker: &str, payment_decimals: u8) -> Result<MarketPrice, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.price_at(ticker, payment_decimals, now)
    }
}

/// Seconds between simulated price ticks.
pub const PRICE_TICK_SECS: u64 = 5;

/// A simulated live market. Each ticker has a stable base price (₦25–₦250)
/// and moves around it along three seeded waves (days, hours, minutes) plus
/// per-tick jitter, bounded to about ±15%. Prices are a pure function of
/// ticker and time, so history is exact after restarts and never stored.
/// Prices are generated, never supplied by an issuer.
pub struct SimulatedPriceFeed;

/// Wave periods in seconds and amplitudes as a fraction of the base price.
const WAVES: [(f64, f64); 3] = [(6.0 * 86_400.0, 0.08), (19.0 * 3_600.0, 0.035), (47.0 * 60.0, 0.012)];
const JITTER: f64 = 0.004;

fn seed_bytes(input: &[u8]) -> [u8; 32] {
    Sha256::digest(input).into()
}

fn unit(bytes: &[u8]) -> f64 {
    let value = u32::from_be_bytes(bytes[..4].try_into().expect("4 bytes"));
    f64::from(value) / f64::from(u32::MAX)
}

impl PriceFeed for SimulatedPriceFeed {
    fn price_at(&self, ticker: &str, payment_decimals: u8, at: u64) -> Result<MarketPrice, String> {
        if payment_decimals < 2 || payment_decimals > 12 {
            return Err("payment asset decimals are unsupported".into());
        }
        let ticker = ticker.trim().to_ascii_uppercase();
        let seed = seed_bytes(ticker.as_bytes());
        let base = 25 + u64::from_be_bytes(seed[..8].try_into().expect("8 bytes")) % 226;

        let tick = at / PRICE_TICK_SECS;
        let time = (tick * PRICE_TICK_SECS) as f64;
        let mut offset = 0.0;
        for (index, (period, amplitude)) in WAVES.iter().enumerate() {
            // Per-ticker period stretch (0.8–1.2×) and phase.
            let stretch = 0.8 + 0.4 * unit(&seed[8 + index * 4..]);
            let phase = std::f64::consts::TAU * unit(&seed[20 + index * 4..]);
            offset += amplitude * (std::f64::consts::TAU * time / (period * stretch) + phase).sin();
        }
        let jitter = seed_bytes(format!("{ticker}:{tick}").as_bytes());
        offset += JITTER * (2.0 * unit(&jitter) - 1.0);

        let cents = (base as f64 * 100.0 * (1.0 + offset)).round().max(1.0) as u64;
        let price_base_units = cents
            .checked_mul(10_u64.pow(u32::from(payment_decimals) - 2))
            .ok_or_else(|| "simulated market price overflowed".to_string())?;
        Ok(MarketPrice {
            price_base_units,
            source: "rabovel_simulated_market_feed".into(),
            as_of: tick * PRICE_TICK_SECS,
        })
    }
}

/// Reference prices for the matching engine: only instruments in the catalog
/// (NGX-listed or live issuer listings) are tradable, and each is priced by
/// the configured feed.
pub struct CatalogReferencePrices {
    instruments: Arc<dyn InstrumentProvider>,
    feed: Arc<dyn PriceFeed>,
}

impl CatalogReferencePrices {
    pub fn new(instruments: Arc<dyn InstrumentProvider>, feed: Arc<dyn PriceFeed>) -> Self {
        Self { instruments, feed }
    }
}

#[async_trait]
impl ReferencePriceSource for CatalogReferencePrices {
    async fn reference_price(
        &self,
        symbol: &str,
        price_decimals: u8,
    ) -> Result<Option<ReferencePrice>, String> {
        let Some(instrument) = self.instruments.find(symbol).await? else {
            return Ok(None);
        };
        let price = self.feed.latest(&instrument.ticker, price_decimals)?;
        Ok(Some(ReferencePrice {
            price_base_units: price.price_base_units,
            as_of: price.as_of,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instrument_provider::SimulatedInstrumentProvider;

    #[tokio::test]
    async fn catalog_reference_prices_only_quote_catalog_instruments() {
        let prices = CatalogReferencePrices::new(
            Arc::new(SimulatedInstrumentProvider),
            Arc::new(SimulatedPriceFeed),
        );
        let quoted = prices.reference_price("dangcem", 6).await.unwrap().unwrap();
        assert_eq!(
            quoted.price_base_units,
            SimulatedPriceFeed
                .price_at("DANGCEM", 6, quoted.as_of)
                .unwrap()
                .price_base_units
        );
        assert_eq!(prices.reference_price("NOPE", 6).await.unwrap(), None);
    }

    #[test]
    fn simulated_prices_are_deterministic_bounded_and_scaled_to_payment_decimals() {
        let feed = SimulatedPriceFeed;
        let at = 1_790_000_003;
        let first = feed.price_at("RABOTELCO", 6, at).unwrap();
        let replay = feed.price_at("rabotelco", 6, at - 3).unwrap();
        // Same 5-second tick, any casing: identical price and tick time.
        assert_eq!(first.price_base_units, replay.price_base_units);
        assert_eq!(first.as_of, 1_790_000_000);
        // Whole cents.
        assert_eq!(first.price_base_units % 10_000, 0);
        assert!((21_000_000..=290_000_000).contains(&first.price_base_units));
        assert_eq!(
            feed.price_at("RABOTELCO", 2, at).unwrap().price_base_units * 10_000,
            first.price_base_units
        );
    }

    #[test]
    fn simulated_prices_move_over_time_within_fifteen_percent() {
        let feed = SimulatedPriceFeed;
        let samples: Vec<u64> = (0..2_000)
            .map(|step| {
                feed.price_at("DANGCEM", 6, 1_790_000_000 + step * 600)
                    .unwrap()
                    .price_base_units
            })
            .collect();
        let low = *samples.iter().min().unwrap();
        let high = *samples.iter().max().unwrap();
        assert!(high > low, "prices must move");
        assert!(high as f64 / low as f64 <= 1.15 / 0.85 + 0.01);
    }
}
