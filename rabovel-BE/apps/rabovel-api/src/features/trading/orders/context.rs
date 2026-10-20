use crate::features::trading::TradingUseCases;
use crate::ApiError;
use crate::AuthenticatedUser;
use crate::GatewayState;
use axum::extract::FromRef;
use identity::AuthError;
use portfolio_trading::OrderActor;
use portfolio_trading::OrderWorkflowError;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct TradingHttpState {
    pub(super) orders: Arc<TradingUseCases>,
}
impl FromRef<GatewayState> for TradingHttpState {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            orders: state.order_use_cases.clone(),
        }
    }
}
impl From<AuthenticatedUser> for OrderActor {
    fn from(user: AuthenticatedUser) -> Self {
        Self {
            user_id: user.user_id,
            mfa_verified_at: user.mfa_verified_at,
        }
    }
}
impl From<OrderWorkflowError<AuthError>> for ApiError {
    fn from(error: OrderWorkflowError<AuthError>) -> Self {
        match error {
            OrderWorkflowError::Access(error) => error.into(),
            OrderWorkflowError::Trading(error) => error.into(),
        }
    }
}
