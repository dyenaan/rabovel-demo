use super::context::TradingHttpState;
use crate::ApiError;
use crate::AuthenticatedUser;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use portfolio_trading::NewOrderRequest;
use portfolio_trading::OrderAck;
use portfolio_trading::OrderStatus;

pub(crate) async fn place_order(
    State(state): State<TradingHttpState>,
    user: AuthenticatedUser,
    Json(request): Json<NewOrderRequest>,
) -> Result<Json<OrderAck>, ApiError> {
    Ok(Json(state.orders.place_order(&user.into(), request).await?))
}
pub(crate) async fn get_order(
    State(state): State<TradingHttpState>,
    user: AuthenticatedUser,
    Path(order_id): Path<String>,
) -> Result<Json<OrderStatus>, ApiError> {
    Ok(Json(state.orders.get_order(&user.into(), &order_id).await?))
}
pub(crate) async fn list_orders(
    State(state): State<TradingHttpState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<OrderStatus>>, ApiError> {
    Ok(Json(state.orders.list_orders(&user.into()).await?))
}
pub(crate) async fn cancel_order(
    State(state): State<TradingHttpState>,
    user: AuthenticatedUser,
    Path(order_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.orders.cancel_order(&user.into(), &order_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
