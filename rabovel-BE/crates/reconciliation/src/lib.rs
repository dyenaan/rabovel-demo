//! `reconciliation` — stub wiring only.
//!
//! This crate proves the "plug and play" Kafka wiring works end-to-end: it
//! connects to the shared `events` bus and subscribes to the trade,
//! portfolio, and tokenization-completed topics it needs to eventually
//! reconcile. It deliberately contains **no** reconciliation business logic
//! — that lives in a colleague's separate implementation. The real behavior
//! is in `main.rs`.

/// Identifies this service in startup/log lines.
pub const SERVICE_NAME: &str = "reconciliation";
