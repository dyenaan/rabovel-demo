use crate::features::market_data::charts::dto::NavPointResponse;
use crate::features::market_data::charts::dto::PricePointResponse;
use crate::features::market_data::charts::usecases::RangeQuery;
use crate::http::errors::ApiError;
use crate::http::models::Data;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Json;

pub(crate) async fn market_price_history_handler(
    State(state): State<super::context::FeatureContext>,
    Path(market_id): Path<String>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<Data<Vec<PricePointResponse>>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::market_price_history(&state, market_id, query).await?,
    }))
}

pub(crate) async fn asset_nav_history_handler(
    State(state): State<super::context::FeatureContext>,
    Path(asset_id): Path<String>,
) -> Result<Json<Data<Vec<NavPointResponse>>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::asset_nav_history(&state, asset_id).await?,
    }))
}
