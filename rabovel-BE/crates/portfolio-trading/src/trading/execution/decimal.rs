//! Exact conversion between API decimal strings and integer base units.
//! Never rounds: input with more fractional digits than the scale allows is
//! rejected rather than silently truncated.

/// Parses a non-negative decimal string (e.g. `"100.25"`) into base units at
/// `decimals` fractional digits. Trailing fractional zeros beyond the scale
/// are accepted (`"10.000"` at scale 0 is `10`).
pub(crate) fn parse_fixed(input: &str, decimals: u8) -> Option<u64> {
    let input = input.trim();
    let (whole, fraction) = input.split_once('.').unwrap_or((input, ""));
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    if !whole.bytes().all(|b| b.is_ascii_digit()) || !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let fraction = fraction.trim_end_matches('0');
    if fraction.len() > usize::from(decimals) {
        return None;
    }
    let scale = 10_u64.checked_pow(decimals.into())?;
    let whole: u64 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };
    let padded = format!("{fraction:0<width$}", width = usize::from(decimals));
    let fraction: u64 = if padded.is_empty() {
        0
    } else {
        padded.parse().ok()?
    };
    whole.checked_mul(scale)?.checked_add(fraction)
}

/// Formats base units as a minimal decimal string (`100500000`, 6 → `"100.5"`).
pub(crate) fn format_fixed(value: u128, decimals: u8) -> String {
    let scale = 10_u128.pow(decimals.into());
    let whole = value / scale;
    let fraction = value % scale;
    if fraction == 0 {
        return whole.to_string();
    }
    let fraction = format!("{fraction:0>width$}", width = usize::from(decimals));
    format!("{whole}.{}", fraction.trim_end_matches('0'))
}

/// Signed variant of [`format_fixed`] for net balances.
pub(crate) fn format_signed(value: i128, decimals: u8) -> String {
    let formatted = format_fixed(value.unsigned_abs(), decimals);
    if value < 0 {
        format!("-{formatted}")
    } else {
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exact_values_and_rejects_excess_precision() {
        assert_eq!(parse_fixed("100", 6), Some(100_000_000));
        assert_eq!(parse_fixed("100.25", 6), Some(100_250_000));
        assert_eq!(parse_fixed(".5", 6), Some(500_000));
        assert_eq!(parse_fixed("10.000", 0), Some(10));
        assert_eq!(parse_fixed(" 7 ", 0), Some(7));
        assert_eq!(parse_fixed("1.5", 0), None);
        assert_eq!(parse_fixed("0.0000001", 6), None);
        for bad in ["", ".", "-1", "1e3", "abc", "1.2.3", "NaN"] {
            assert_eq!(parse_fixed(bad, 6), None, "{bad}");
        }
        assert_eq!(parse_fixed("99999999999999999999", 0), None);
    }

    #[test]
    fn formats_minimal_decimal_strings() {
        assert_eq!(format_fixed(100_500_000, 6), "100.5");
        assert_eq!(format_fixed(100_000_000, 6), "100");
        assert_eq!(format_fixed(1, 6), "0.000001");
        assert_eq!(format_fixed(42, 0), "42");
        assert_eq!(format_signed(-1_500_000, 6), "-1.5");
    }
}
