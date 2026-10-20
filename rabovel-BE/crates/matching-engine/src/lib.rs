//! Deterministic I/O-free matching rules; callers own clocks, persistence and delivery.

mod book;
mod house;
mod orders;
pub use house::{
    HedgeInstruction, HedgeSettlement, HouseLimits, HousePosition, HouseQuote, InventoryMovement,
};
pub use orders::*;
#[cfg(test)]
mod tests;
