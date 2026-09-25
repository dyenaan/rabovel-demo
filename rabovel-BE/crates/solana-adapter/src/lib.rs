//! Token-2022 equity setup and Token ACL foundation.
//! See demo/ONCHAIN_REFERENCE.md for implemented and pending capabilities.

pub mod authority_config;
mod error;
pub mod token_acl;

#[path = "EquitySetupService/mod.rs"]
pub mod equity_setup_service;

pub use equity_setup_service::{EquitySetupService, token_mint_builder};
pub use error::IssuanceError;
