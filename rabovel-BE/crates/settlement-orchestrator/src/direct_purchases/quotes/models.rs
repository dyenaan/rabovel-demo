//! Current direct-purchase pricing policy; amounts remain exact base units.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuoteConfig {
    pub fee_bps: u32,
    pub expiry_seconds: u64,
}
impl Default for QuoteConfig {
    fn default() -> Self {
        Self {
            fee_bps: 100,
            expiry_seconds: 300,
        }
    }
}
impl QuoteConfig {
    pub fn parse(fee: Option<&str>, expiry: Option<&str>) -> Result<Self, &'static str> {
        Ok(Self {
            fee_bps: fee
                .unwrap_or("100")
                .parse()
                .map_err(|_| "invalid demo fee configuration")?,
            expiry_seconds: expiry
                .unwrap_or("300")
                .parse()
                .map_err(|_| "invalid demo quote expiry configuration")?,
        })
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct PurchaseAmounts {
    pub cost: u64,
    pub fee: u64,
    pub total: u64,
}
pub fn buy_amounts(
    quantity: u64,
    unit_price: u64,
    fee_bps: u32,
) -> Result<PurchaseAmounts, &'static str> {
    let cost = quantity
        .checked_mul(unit_price)
        .ok_or("quantity overflow")?;
    let fee = cost
        .checked_mul(u64::from(fee_bps))
        .ok_or("fee calculation overflow")?
        / 10_000;
    let total = cost.checked_add(fee).ok_or("total payment overflow")?;
    Ok(PurchaseAmounts { cost, fee, total })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_buy_amounts_keep_fee_rounding_and_distinguish_overflows() {
        assert_eq!(
            buy_amounts(3, 101, 100),
            Ok(PurchaseAmounts {
                cost: 303,
                fee: 3,
                total: 306
            })
        );
        assert_eq!(buy_amounts(u64::MAX, 2, 0), Err("quantity overflow"));
        assert_eq!(buy_amounts(1, u64::MAX, 2), Err("fee calculation overflow"));
        assert_eq!(buy_amounts(1, u64::MAX, 1), Err("total payment overflow"));
    }
    #[test]
    fn configuration_defaults_and_invalid_inputs_match_existing_policy() {
        assert_eq!(QuoteConfig::parse(None, None), Ok(QuoteConfig::default()));
        assert!(QuoteConfig::parse(Some("bad"), None).is_err());
        assert!(QuoteConfig::parse(None, Some("-1")).is_err());
        assert_eq!(
            QuoteConfig::parse(Some("0"), Some("0"))
                .unwrap()
                .expiry_seconds,
            0
        );
    }
}
