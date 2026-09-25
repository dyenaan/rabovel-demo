use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct MarketPrice {
    pub price_base_units: u64,
    pub source: String,
    pub as_of: u64,
}

pub trait PriceFeed: Send + Sync {
    fn latest(&self, ticker: &str, payment_decimals: u8) -> Result<MarketPrice, String>;
}

/// Stable simulated market data for the demo. Prices are generated from the
/// ticker, never supplied by an issuer, and remain consistent across restarts.
pub struct SimulatedPriceFeed;

impl PriceFeed for SimulatedPriceFeed {
    fn latest(&self, ticker: &str, payment_decimals: u8) -> Result<MarketPrice, String> {
        let scale = 10_u64
            .checked_pow(payment_decimals.into())
            .ok_or_else(|| "payment asset decimals are unsupported".to_string())?;
        let digest = Sha256::digest(ticker.trim().to_ascii_uppercase().as_bytes());
        let seed = u64::from_be_bytes(digest[..8].try_into().expect("SHA-256 prefix is 8 bytes"));
        let whole_cngn = 25 + seed % 226;
        let price_base_units = whole_cngn
            .checked_mul(scale)
            .ok_or_else(|| "simulated market price overflowed".to_string())?;
        Ok(MarketPrice {
            price_base_units,
            source: "rabovel_simulated_market_feed".into(),
            as_of: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated_prices_are_stable_per_ticker_and_scaled_to_payment_decimals() {
        let feed = SimulatedPriceFeed;
        let first = feed.latest("RABO-NG-TELCO", 6).unwrap();
        let replay = feed.latest("rabo-ng-telco", 6).unwrap();
        assert_eq!(first.price_base_units, replay.price_base_units);
        assert_eq!(first.price_base_units % 1_000_000, 0);
        assert!((25_000_000..=250_000_000).contains(&first.price_base_units));
    }
}
