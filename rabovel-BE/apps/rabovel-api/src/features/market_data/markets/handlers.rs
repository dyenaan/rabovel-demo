use crate::features::market_data::markets::dto::MarketDetailResponse;
use crate::features::market_data::markets::dto::MarketListResponse;
use crate::features::market_data::markets::dto::OrderBookResponse;
use crate::features::market_data::markets::dto::PublicTradeResponse;
use crate::http::errors::ApiError;
use crate::http::models::Data;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Json;

#[utoipa::path(get, path = "/markets", responses((status = 200, body = MarketListResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn list_markets_handler(
    State(state): State<super::context::FeatureContext>,
) -> Result<Json<MarketListResponse>, ApiError> {
    Ok(Json(super::usecases::list_markets(&state).await?))
}

#[utoipa::path(get, path = "/markets/{market_id}", responses((status = 200, body = MarketDetailResponse), (status = 404, body = ApiErrorResponse), (status = 503, body = ApiErrorResponse)))]
pub(crate) async fn get_market_handler(
    State(state): State<super::context::FeatureContext>,
    Path(market_id): Path<String>,
) -> Result<Json<MarketDetailResponse>, ApiError> {
    Ok(Json(super::usecases::get_market(&state, market_id).await?))
}

pub(crate) async fn market_order_book_handler(
    State(state): State<super::context::FeatureContext>,
    Path(market_id): Path<String>,
) -> Result<Json<Data<OrderBookResponse>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::market_order_book(&state, market_id).await?,
    }))
}

pub(crate) async fn market_trades_handler(
    State(state): State<super::context::FeatureContext>,
    Path(market_id): Path<String>,
) -> Result<Json<Data<Vec<PublicTradeResponse>>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::market_trades(&state, market_id).await?,
    }))
}
