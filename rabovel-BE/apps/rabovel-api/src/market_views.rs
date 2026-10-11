//! Pure helpers that turn engine data into the frontend's market, asset and
//! portfolio views: price candles, fixed-point display strings and times.

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const HOUR: u64 = 3_600;
const DAY: u64 = 86_400;

/// Bucket size and count for a chart range such as `1D`, `1W`, `1M`, `1Y`.
pub fn range_buckets(range: &str) -> (u64, usize) {
    match range.trim().to_ascii_uppercase().as_str() {
        "1D" => (HOUR, 24),
        "1W" => (HOUR, 168),
        "1M" => (DAY, 30),
        "3M" => (DAY, 90),
        "6M" => (DAY, 180),
        "1Y" | "YTD" | "ALL" => (DAY, 365),
        _ => (HOUR, 120),
    }
}

/// One OHLC bucket in payment-asset base units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candle {
    pub start: u64,
    pub open: u64,
    pub high: u64,
    pub low: u64,
    pub close: u64,
    pub volume: u64,
}

/// A matched trade as `(executed_at, price_base_units, quantity)`.
pub type TradePoint = (u64, u64, u64);

/// Reference-price samples taken inside each candle.
const SAMPLES_PER_CANDLE: u64 = 12;

/// Candles ending at the bucket that contains `now`. Open, close, high and
/// low sample the reference price path (`price_at`) inside each bucket, never
/// past `now`; engine trades add volume and widen the high/low.
pub fn candles(
    trades: &[TradePoint],
    price_at: impl Fn(u64) -> u64,
    now: u64,
    bucket: u64,
    count: usize,
) -> Vec<Candle> {
    let end = (now / bucket + 1) * bucket;
    let start = end.saturating_sub(bucket * count as u64);
    (0..count as u64)
        .map(|index| {
            let from = start + index * bucket;
            let last = (from + bucket - 1).min(now);
            let step = (bucket / SAMPLES_PER_CANDLE).max(1);
            let open = price_at(from);
            let close = price_at(last);
            let mut candle = Candle {
                start: from,
                open,
                high: open.max(close),
                low: open.min(close),
                close,
                volume: 0,
            };
            for sample in (from..=last).step_by(step as usize).map(&price_at) {
                candle.high = candle.high.max(sample);
                candle.low = candle.low.min(sample);
            }
            for (_, price, quantity) in trades
                .iter()
                .filter(|(at, _, _)| (from..from + bucket).contains(at))
            {
                candle.high = candle.high.max(*price);
                candle.low = candle.low.min(*price);
                candle.volume += quantity;
            }
            candle
        })
        .collect()
}

/// Signed base units as a two-decimal display amount, rounded half away
/// from zero (e.g. `-1234567` at 6 decimals → `"-1.23"`).
pub fn fixed2(value: i128, decimals: u8) -> String {
    let scale = 10_i128.pow(u32::from(decimals));
    let cents = (value.abs() * 100 + scale / 2) / scale;
    let sign = if value < 0 && cents > 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", cents / 100, cents % 100)
}

/// `numerator / denominator` as a two-decimal percentage; `0.00` when the
/// denominator is zero.
pub fn percent(numerator: i128, denominator: i128) -> String {
    if denominator == 0 {
        return "0.00".into();
    }
    let basis = numerator * 10_000 / denominator;
    let sign = if basis < 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", basis.abs() / 100, basis.abs() % 100)
}

/// Parses a non-negative decimal string into base units, truncating extra
/// fraction digits (`"100.5"` at 6 decimals → `100_500_000`).
pub fn parse_units(value: &str, decimals: u8) -> Option<i128> {
    let (whole, fraction) = value.trim().split_once('.').unwrap_or((value.trim(), ""));
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    let digits = usize::from(decimals);
    let fraction: String = fraction.chars().chain(std::iter::repeat('0')).take(digits).collect();
    let whole: i128 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
    let fraction: i128 = if digits == 0 { 0 } else { fraction.parse().ok()? };
    Some(whole * 10_i128.pow(u32::from(decimals)) + fraction)
}

/// Unix seconds as RFC 3339.
pub fn iso(timestamp: u64) -> String {
    OffsetDateTime::from_unix_timestamp(timestamp as i64)
        .ok()
        .and_then(|time| time.format(&Rfc3339).ok())
        .unwrap_or_default()
}

/// Unix seconds as a calendar date (`YYYY-MM-DD`).
pub fn iso_date(timestamp: u64) -> String {
    iso(timestamp).chars().take(10).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candles_sample_the_price_path_and_trades_add_volume() {
        // Price rises 1 per second.
        let path = |at: u64| 1_000 + at;
        let now = 10 * HOUR + 60;
        let trades = [(9 * HOUR + 10, 5_000, 2), (9 * HOUR + 20, 900, 1), (8 * HOUR, 9_100, 5)];
        let series = candles(&trades, path, now, HOUR, 4);
        assert_eq!(series.len(), 4);
        // Buckets 7h, 8h, 9h, 10h.
        assert_eq!(series[0].start, 7 * HOUR);
        assert_eq!(series[0].open, path(7 * HOUR));
        assert_eq!(series[0].close, path(8 * HOUR - 1));
        assert_eq!(series[0].volume, 0);
        assert_eq!(series[1].volume, 5);
        // Trades outside the path widen the range but not open/close.
        assert_eq!((series[2].high, series[2].low, series[2].volume), (path(10 * HOUR - 1), 900, 3));
        assert_eq!(series[2].close, path(10 * HOUR - 1));
        // The current bucket never samples the future.
        assert_eq!(series[3].close, path(now));
        assert_eq!(series[3].high, path(now));
    }

    #[test]
    fn fixed_point_display_and_parsing_round_trip() {
        assert_eq!(fixed2(100_500_000, 6), "100.50");
        assert_eq!(fixed2(-1_234_567, 6), "-1.23");
        assert_eq!(fixed2(4_999, 6), "0.00");
        assert_eq!(fixed2(5_000, 6), "0.01");
        assert_eq!(parse_units("100.5", 6), Some(100_500_000));
        assert_eq!(parse_units("7", 0), Some(7));
        assert_eq!(parse_units("1.2345678", 6), Some(1_234_567));
        assert_eq!(parse_units("abc", 6), None);
        assert_eq!(percent(-5, 200), "-2.50");
        assert_eq!(percent(1, 0), "0.00");
        assert_eq!(iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_date(DAY + 5), "1970-01-02");
    }
}
