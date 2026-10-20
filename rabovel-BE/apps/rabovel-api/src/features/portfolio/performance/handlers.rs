use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::market_data::charts::usecases::RangeQuery;
use crate::features::portfolio::performance::dto::PerformancePointResponse;
use crate::http::errors::ApiError;
use crate::http::models::Data;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Json;

/// Current holdings and cash valued at each bucket's closing price. Engine
/// state has no history of past positions, so earlier points show today's
/// holdings at earlier prices.
pub(crate) async fn portfolio_performance_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
    Query(query): Query<RangeQuery>,
) -> Result<Json<Data<Vec<PerformancePointResponse>>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::portfolio_performance(&state, user, query).await?,
    }))
}
