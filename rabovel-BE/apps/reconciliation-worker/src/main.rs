//! Minimal Kafka consumer wiring for `reconciliation`.
//!
//! Subscribes to [`events::topics::TRADING_TRADE_EXECUTED_V1`],
//! [`events::topics::PORTFOLIO_CHANGED_V1`], and
//! [`events::topics::TOKENIZATION_COMPLETED_V1`], logging each received
//! event. No reconciliation business logic lives here — this only proves the
//! consumer plumbing works end-to-end. The shared connect/decode loop is
//! [`infrastructure::messaging::consumer::log_events`].

use events::schema::tokenization::TokenizationCompleted;
use events::schema::trading::{PortfolioChanged, TradeExecuted};
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
        reconciliation::SERVICE_NAME,
        bootstrap_servers
    );

    let trade_executed = tokio::spawn(log_events::<TradeExecuted>(
        bootstrap_servers.clone(),
        topics::TRADING_TRADE_EXECUTED_V1,
    ));
    let portfolio_changed = tokio::spawn(log_events::<PortfolioChanged>(
        bootstrap_servers.clone(),
        topics::PORTFOLIO_CHANGED_V1,
    ));
    let tokenization_completed = tokio::spawn(log_events::<TokenizationCompleted>(
        bootstrap_servers,
        topics::TOKENIZATION_COMPLETED_V1,
    ));

    let _ = tokio::join!(trade_executed, portfolio_changed, tokenization_completed);
}
