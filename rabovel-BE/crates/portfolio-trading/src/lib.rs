//! Trading workflows, execution ports and portfolio queries.

mod portfolio;
mod trading;
pub use matching_engine::{EngineConfig, HouseLimits, InventoryMovement};
pub use portfolio::balances::{models::*, ports::BalanceQueries};
pub(crate) use trading::execution::clock::now_secs;
pub use trading::execution::engine_service::{
    EngineTradingConfig, EngineTradingService, ReferencePrice, ReferencePriceSource,
};
pub use trading::market_queries::ports::MarketQueries;
pub use trading::orders::models::*;
pub use trading::orders::ports::*;
pub use trading::orders::usecases::{OrderUseCases, OrderWorkflowError};
pub use trading::repricing::ports::Repricing;
pub use trading::testing::stub::StubPortfolioTradingService;
/// Composition compatibility: application workflows consume the narrower supertraits.
pub trait PortfolioTradingApi:
    OrderCommands + OrderQueries + Repricing + MarketQueries + BalanceQueries
{
}
impl<T: OrderCommands + OrderQueries + Repricing + MarketQueries + BalanceQueries + ?Sized>
    PortfolioTradingApi for T
{
}
