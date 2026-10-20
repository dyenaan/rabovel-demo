use super::*;
use crate::app::persistence::Persistence;
use std::sync::Mutex;

#[derive(Default)]
struct LimitProbe {
    calls: Mutex<Vec<(String, u32, u64)>>,
    reject: bool,
}

#[async_trait]
impl RateLimiter for LimitProbe {
    async fn check(&self, key: &str, limit: u32, seconds: u64) -> Result<(), AuthError> {
        self.calls
            .lock()
            .unwrap()
            .push((key.into(), limit, seconds));
        if self.reject {
            Err(AuthError::RateLimited)
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn place_checks_existing_limiter_key_before_financial_access() {
    let limiter = Arc::new(LimitProbe {
        reject: true,
        ..LimitProbe::default()
    });
    let access = OrderAuthorization {
        accounts: Persistence::memory().accounts,
        wallets: Persistence::memory().wallets,
        auth_service: Arc::new(BrokerageAuthService::new(Default::default())),
        rate_limiter: limiter.clone(),
    };
    let actor = OrderActor {
        user_id: "missing-user".into(),
        mfa_verified_at: None,
    };
    // An absent user would reject as Unauthorized if authorization ran first.
    assert_eq!(
        access.authorize(&actor, OrderAction::Place).await,
        Err(AuthError::RateLimited)
    );
    assert_eq!(
        *limiter.calls.lock().unwrap(),
        vec![("session:missing-user".into(), 10, 60)]
    );
    for action in [OrderAction::View, OrderAction::Cancel] {
        assert_eq!(
            access.authorize(&actor, action).await,
            Err(AuthError::Unauthorized)
        );
    }
    // Reads and cancellation do not consume the place-order quota.
    assert_eq!(limiter.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn allowed_rate_limit_still_requires_current_identity() {
    let limiter = Arc::new(LimitProbe::default());
    let access = OrderAuthorization {
        accounts: Persistence::memory().accounts,
        wallets: Persistence::memory().wallets,
        auth_service: Arc::new(BrokerageAuthService::new(Default::default())),
        rate_limiter: limiter.clone(),
    };
    assert_eq!(
        access
            .authorize(
                &OrderActor {
                    user_id: "missing-user".into(),
                    mfa_verified_at: Some(42),
                },
                OrderAction::Place
            )
            .await,
        Err(AuthError::Unauthorized)
    );
    assert_eq!(limiter.calls.lock().unwrap().len(), 1);
}
