//! Rabovel's principal desk: the guaranteed counterparty of last resort.
//!
//! The house quotes around an external reference price, absorbs client flow
//! the book cannot match, and accumulates that exposure into a hedge window.
//! Closing the window nets every principal fill per symbol into one
//! instruction for the clearing house (T+N), instead of hedging per trade.
//!
//! Token inventory is hybrid: house sales are delivered from the pre-funded
//! treasury float first and only the shortfall is minted against the pending
//! hedge ("minted unbacked" until the clearing buy settles). House purchases
//! retire outstanding unbacked units first and refill the float with the rest.

use serde::{Deserialize, Serialize};

use crate::{Price, Qty, Side, UnfilledReason};

const BPS_DENOMINATOR: u128 = 10_000;

/// Principal-desk limits for one symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseLimits {
    /// Half of the bid/ask spread around the reference price, in basis points.
    pub half_spread_bps: u32,
    /// Largest net long position the house will take by buying from clients.
    pub max_long: Qty,
    /// Largest net short position the house will take by selling to clients.
    pub max_short: Qty,
    /// Largest number of tokens that may exist minted ahead of the underlying
    /// being acquired. `0` means delivery is strictly from the float.
    pub max_minted_unbacked: Qty,
    /// Largest quantity a single client order may request.
    pub max_order_quantity: Qty,
}

impl Default for HouseLimits {
    fn default() -> Self {
        Self {
            half_spread_bps: 50,
            max_long: 100_000,
            max_short: 100_000,
            max_minted_unbacked: 100_000,
            max_order_quantity: 10_000,
        }
    }
}

/// The house's two-sided price for one symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseQuote {
    /// Price at which the house buys from clients; `None` when it rounds to 0.
    pub bid: Option<Price>,
    pub ask: Price,
}

impl HouseQuote {
    pub fn from_reference(reference: Price, half_spread_bps: u32) -> Option<Self> {
        let reference = u128::from(reference);
        let spread = u128::from(half_spread_bps).min(BPS_DENOMINATOR);
        let bid = reference * (BPS_DENOMINATOR - spread) / BPS_DENOMINATOR;
        let ask = (reference * (BPS_DENOMINATOR + spread)).div_ceil(BPS_DENOMINATOR);
        Some(Self {
            bid: u64::try_from(bid).ok().filter(|bid| *bid > 0),
            ask: u64::try_from(ask).ok().filter(|ask| *ask > 0)?,
        })
    }

    /// The price a client taking `client_side` would trade at.
    pub fn price_for(&self, client_side: Side) -> Option<Price> {
        match client_side {
            Side::Buy => Some(self.ask),
            Side::Sell => self.bid,
        }
    }
}

/// Snapshot of the house's exposure and token inventory in one symbol.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HousePosition {
    /// Net principal flow: positive = house bought more than it sold.
    pub net_quantity: i64,
    /// Cash received minus cash paid, in payment-asset base units.
    pub net_cash: i128,
    /// Backed tokens held in the treasury, available for instant delivery.
    pub float: Qty,
    /// Tokens delivered to clients by minting ahead of the clearing buy.
    pub minted_unbacked: Qty,
}

/// How the house side of one principal fill moves tokens on-chain. The
/// settlement worker turns this into transfer/mint/burn instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum InventoryMovement {
    /// House sold: transfer `from_float` out of the treasury, mint `minted`
    /// to the buyer against the pending hedge.
    HouseDelivers { from_float: Qty, minted: Qty },
    /// House bought: burn `retired` (cancelling unbacked supply), keep
    /// `to_float` in the treasury.
    HouseReceives { to_float: Qty, retired: Qty },
}

/// Result of a clearing-house hedge settling (T+N).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HedgeSettlement {
    /// Previously minted-unbacked tokens that are now backed.
    pub backed: Qty,
    /// Units added to (buy) or removed from (sell) the float.
    pub float_change: Qty,
}

#[derive(Debug, Clone, Default)]
struct HedgeWindow {
    bought: Qty,
    buy_notional: u128,
    sold: Qty,
    sell_notional: u128,
    minted: Qty,
    fills: u32,
}

#[derive(Debug, Default)]
pub(crate) struct HouseState {
    pub position: HousePosition,
    window: HedgeWindow,
}

impl HouseState {
    fn risk_capacity(&self, client_side: Side, limits: &HouseLimits) -> Qty {
        let net = i128::from(self.position.net_quantity);
        let capacity = match client_side {
            Side::Buy => net + i128::from(limits.max_short),
            Side::Sell => i128::from(limits.max_long) - net,
        };
        u64::try_from(capacity.max(0)).unwrap_or(u64::MAX)
    }

    fn inventory_capacity(&self, client_side: Side, limits: &HouseLimits) -> Qty {
        match client_side {
            Side::Buy => {
                let mint_headroom = limits
                    .max_minted_unbacked
                    .saturating_sub(self.position.minted_unbacked);
                self.position.float.saturating_add(mint_headroom)
            }
            // Receiving tokens never runs out of inventory.
            Side::Sell => Qty::MAX,
        }
    }

    /// How many units the house can still trade against a client on `client_side`.
    pub fn capacity(&self, client_side: Side, limits: &HouseLimits) -> Qty {
        self.risk_capacity(client_side, limits)
            .min(self.inventory_capacity(client_side, limits))
    }

    /// Why [`Self::capacity`] is zero.
    pub fn exhausted_reason(&self, client_side: Side, limits: &HouseLimits) -> UnfilledReason {
        if self.risk_capacity(client_side, limits) == 0 {
            UnfilledReason::HouseRiskLimit
        } else {
            UnfilledReason::InventoryLimit
        }
    }

    pub fn record(&mut self, client_side: Side, quantity: Qty, price: Price) -> InventoryMovement {
        let notional = u128::from(quantity) * u128::from(price);
        let signed_quantity = i64::try_from(quantity).unwrap_or(i64::MAX);
        let position = &mut self.position;
        self.window.fills += 1;
        match client_side {
            Side::Buy => {
                position.net_quantity -= signed_quantity;
                position.net_cash += notional as i128;
                self.window.sold += quantity;
                self.window.sell_notional += notional;
                let from_float = quantity.min(position.float);
                let minted = quantity - from_float;
                position.float -= from_float;
                position.minted_unbacked += minted;
                self.window.minted += minted;
                InventoryMovement::HouseDelivers { from_float, minted }
            }
            Side::Sell => {
                position.net_quantity += signed_quantity;
                position.net_cash -= notional as i128;
                self.window.bought += quantity;
                self.window.buy_notional += notional;
                let retired = quantity.min(position.minted_unbacked);
                let to_float = quantity - retired;
                position.minted_unbacked -= retired;
                position.float += to_float;
                InventoryMovement::HouseReceives { to_float, retired }
            }
        }
    }

    pub fn fund_float(&mut self, quantity: Qty) {
        self.position.float = self.position.float.saturating_add(quantity);
    }

    /// Applies a settled clearing-house hedge. A settled buy backs unbacked
    /// mints first and adds the rest to the float; a settled sell removes
    /// units from the float. Returns `None` if a sell exceeds the float.
    pub fn settle_hedge(&mut self, hedge_side: Side, quantity: Qty) -> Option<HedgeSettlement> {
        let position = &mut self.position;
        match hedge_side {
            Side::Buy => {
                let backed = quantity.min(position.minted_unbacked);
                position.minted_unbacked -= backed;
                position.float += quantity - backed;
                Some(HedgeSettlement {
                    backed,
                    float_change: quantity - backed,
                })
            }
            Side::Sell => {
                position.float = position.float.checked_sub(quantity)?;
                Some(HedgeSettlement {
                    backed: 0,
                    float_change: quantity,
                })
            }
        }
    }

    pub fn close_window(&mut self, symbol: &str) -> Option<HedgeInstruction> {
        let window = std::mem::take(&mut self.window);
        if window.fills == 0 {
            return None;
        }
        let (hedge_side, hedge_quantity) = match window.sold.cmp(&window.bought) {
            std::cmp::Ordering::Greater => (Some(Side::Buy), window.sold - window.bought),
            std::cmp::Ordering::Less => (Some(Side::Sell), window.bought - window.sold),
            std::cmp::Ordering::Equal => (None, 0),
        };
        Some(HedgeInstruction {
            symbol: symbol.to_string(),
            hedge_side,
            hedge_quantity,
            house_bought: window.bought,
            house_sold: window.sold,
            buy_notional: window.buy_notional,
            sell_notional: window.sell_notional,
            minted_in_window: window.minted,
            minted_unbacked: self.position.minted_unbacked,
            principal_fills: window.fills,
        })
    }
}

/// Net clearing-house action for one symbol over a closed hedge window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HedgeInstruction {
    pub symbol: String,
    /// What the house must do in the underlying market; `None` when client
    /// buys and sells netted to zero.
    pub hedge_side: Option<Side>,
    pub hedge_quantity: Qty,
    pub house_bought: Qty,
    pub house_sold: Qty,
    pub buy_notional: u128,
    pub sell_notional: u128,
    /// Tokens minted against this window's hedge.
    pub minted_in_window: Qty,
    /// All tokens still awaiting backing at close; the clearing buy must
    /// cover at least this many to make every delivered token backed.
    pub minted_unbacked: Qty,
    pub principal_fills: u32,
}
