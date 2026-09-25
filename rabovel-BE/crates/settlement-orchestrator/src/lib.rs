//! `settlement-orchestrator` — stub wiring only.
//!
//! This crate proves the "plug and play" Kafka wiring works end-to-end: it
//! connects to the shared `events` bus and subscribes to the trade and
//! tokenization-completed topics it needs to eventually settle. It
//! deliberately contains **no** settlement business logic — that lives in a
//! colleague's separate implementation. The real behavior is in `main.rs`.

/// Identifies this service in startup/log lines.
pub const SERVICE_NAME: &str = "settlement-orchestrator";
