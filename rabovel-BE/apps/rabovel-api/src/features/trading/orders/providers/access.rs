use crate::features::identity::access::policy::authorize_trading_action;
use crate::SESSION_RATE_LIMIT;
use async_trait::async_trait;
use identity::access::rate_limiter::RateLimiter;
use identity::AuthError;
use identity::BrokerageAuthService;
use identity::WalletAction;
use portfolio_trading::OrderAccessPolicy;
use portfolio_trading::OrderAction;
use portfolio_trading::OrderActor;
use std::sync::Arc;

// Order-specific access policy over identity queries and current rate-limit rules.

pub(in crate::features::trading) struct OrderAuthorization {
    pub accounts: Arc<dyn identity::accounts::ports::AccountQueries>,
    pub(crate) wallets: Arc<dyn identity::wallets::ports::WalletQueries>,
    pub auth_service: Arc<BrokerageAuthService>,
    pub rate_limiter: Arc<dyn RateLimiter>,
}

#[async_trait]
impl OrderAccessPolicy for OrderAuthorization {
    type Error = AuthError;

    async fn authorize(&self, actor: &OrderActor, action: OrderAction) -> Result<(), AuthError> {
        // Keep the historical per-user key and ordering: limit place-order
        // attempts before loading financial authorization, including denials.
        if action == OrderAction::Place {
            self.rate_limiter
                .check(
                    &format!("session:{}", actor.user_id),
                    SESSION_RATE_LIMIT.0,
                    SESSION_RATE_LIMIT.1,
                )
                .await?;
        }
        let permission = match action {
            OrderAction::View => WalletAction::ViewPortfolio,
            OrderAction::Place | OrderAction::Cancel => WalletAction::TradeSpot,
        };
        authorize_trading_action(
            self.accounts.as_ref(),
            self.wallets.as_ref(),
            self.auth_service.as_ref(),
            &actor.user_id,
            actor.mfa_verified_at,
            permission,
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
