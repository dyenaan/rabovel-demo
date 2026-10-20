use crate::app::errors::ApplicationError;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::market_data::charts::calculations as market_views;
use crate::features::portfolio::activity::dto::ActivityEventResponse;
use portfolio_trading::OrderSide;

pub(crate) async fn portfolio_activity(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<Vec<ActivityEventResponse>, ApplicationError> {
    let orders = state
        .order_queries()
        .list_orders(&user.user_id)
        .await
        .map_err(ApplicationError::from)?;
    let mut events = Vec::new();
    for order in orders {
        let side = match order.side {
            OrderSide::Buy => "Buy",
            OrderSide::Sell => "Sell",
        };
        if order.filled_quantity != "0" {
            events.push((
                order.updated_at,
                ActivityEventResponse {
                    event_id: format!("{}_filled", order.order_id),
                    event_type: "ORDER_FILLED".into(),
                    title: format!("{side} {} filled", order.symbol),
                    description: format!(
                        "{} of {} {} at an average of {} NGN",
                        order.filled_quantity,
                        order.quantity,
                        order.symbol,
                        order.average_price.as_deref().unwrap_or("-")
                    ),
                    timestamp: market_views::iso(order.updated_at),
                },
            ));
        }
        events.push((
            order.created_at,
            ActivityEventResponse {
                event_id: format!("{}_placed", order.order_id),
                event_type: "ORDER_PLACED".into(),
                title: format!("{side} order placed"),
                description: format!("{} {} {}", side, order.quantity, order.symbol),
                timestamp: market_views::iso(order.created_at),
            },
        ));
    }
    // Newest first; a fill sorts ahead of its own placement.
    events.sort_by_key(|event| std::cmp::Reverse(event.0));
    Ok(events.into_iter().map(|(_, event)| event).collect())
}
pub(crate) trait Ports: crate::features::trading::orders::ports::Orders {}
impl<T: crate::features::trading::orders::ports::Orders + ?Sized> Ports for T {}
