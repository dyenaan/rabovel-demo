//! `tokenization-orchestrator` — stub wiring only.
//!
//! This crate proves the "plug and play" Kafka wiring works end-to-end: it
//! connects to the shared `events` bus and subscribes to the tokenization
//! request topic. It deliberately contains **no** tokenization business
//! logic (issuance, custody, chain interaction, etc.) — that lives in a
//! colleague's separate implementation. The real behavior is in `main.rs`.

/// Identifies this service in startup/log lines.
pub const SERVICE_NAME: &str = "tokenization-orchestrator";
