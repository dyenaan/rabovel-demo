use super::ports::PriceFeed;
use super::providers::simulated::SimulatedPriceFeed;
use super::*;
use crate::features::catalog::instruments::SimulatedInstrumentProvider;
use portfolio_trading::ReferencePriceSource;
use std::sync::Arc;

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
