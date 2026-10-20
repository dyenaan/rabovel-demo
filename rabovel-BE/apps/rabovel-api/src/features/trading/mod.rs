//! Order feature composition. Shared engine models remain in portfolio-trading.

use events::EventProducer;
use identity::access::rate_limiter::RateLimiter;
use identity::AuthError;
use identity::BrokerageAuthService;
use orders::providers::access::OrderAuthorization;
use orders::providers::events::MessagingTradingEvents;
use portfolio_trading::OrderUseCases;
use portfolio_trading::PortfolioTradingApi;
use std::sync::Arc;

pub(crate) mod orders;

pub(crate) mod repricing;

pub(crate) type TradingUseCases = OrderUseCases<AuthError>;

pub(crate) fn compose(
    engine: Arc<dyn PortfolioTradingApi>,
    accounts: Arc<dyn identity::accounts::ports::AccountRepository>,
    wallets: Arc<dyn identity::wallets::ports::WalletRepository>,
    auth_service: Arc<BrokerageAuthService>,
    rate_limiter: Arc<dyn RateLimiter>,
    producer: Arc<dyn EventProducer>,
) -> Arc<TradingUseCases> {
    Arc::new(OrderUseCases::new(
        engine,
        Arc::new(OrderAuthorization {
            accounts,
            wallets,
            auth_service,
            rate_limiter,
        }),
        Arc::new(MessagingTradingEvents { producer }),
    ))
}

pub(crate) mod ports;
