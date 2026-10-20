#[derive(Debug, Clone)]
pub struct MarketPrice {
    pub price_base_units: u64,
    pub source: String,
    pub as_of: u64,
}
/// Seconds between simulated price ticks.
pub const PRICE_TICK_SECS: u64 = 5;
