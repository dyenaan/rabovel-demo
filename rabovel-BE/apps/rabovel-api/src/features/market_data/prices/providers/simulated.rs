use crate::features::market_data::prices::models::*;
use crate::features::market_data::prices::ports::PriceFeed;
use sha2::Digest;
use sha2::Sha256;

/// A simulated live market. Each ticker has a stable base price (₦25–₦250)
/// and moves around it along three seeded waves (days, hours, minutes) plus
/// per-tick jitter, bounded to about ±15%. Prices are a pure function of
/// ticker and time, so history is exact after restarts and never stored.
/// Prices are generated, never supplied by an issuer.
pub struct SimulatedPriceFeed;
/// Wave periods in seconds and amplitudes as a fraction of the base price.
const WAVES: [(f64, f64); 3] = [
    (6.0 * 86_400.0, 0.08),
    (19.0 * 3_600.0, 0.035),
    (47.0 * 60.0, 0.012),
];
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
        if !(2..=12).contains(&payment_decimals) {
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
