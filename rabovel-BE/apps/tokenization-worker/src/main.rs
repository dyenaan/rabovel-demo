//! Minimal Kafka consumer wiring for `tokenization-orchestrator`.
//!
//! Subscribes to [`events::topics::TOKENIZATION_REQUESTED_V1`] and logs each
//! received event. No tokenization business logic (issuance, custody, chain
//! interaction) lives here — this only proves the consumer plumbing works
//! end-to-end. The shared connect/decode loop is
//! [`infrastructure::messaging::consumer::log_events`].

use events::schema::tokenization::TokenizationRequested;
use events::topics;
use infrastructure::messaging::consumer::{bootstrap_servers_from_env, log_events};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let bootstrap_servers = bootstrap_servers_from_env();

    tracing::info!(
        "{} starting, connecting to {:?}",
        tokenization_orchestrator::SERVICE_NAME,
        bootstrap_servers
    );

    let requested = tokio::spawn(log_events::<TokenizationRequested>(
        bootstrap_servers,
        topics::TOKENIZATION_REQUESTED_V1,
    ));

    let _ = tokio::join!(requested);
}
