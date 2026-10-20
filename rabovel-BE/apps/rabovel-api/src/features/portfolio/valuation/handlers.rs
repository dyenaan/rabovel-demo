use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::portfolio::valuation::dto::HoldingResponse;
use crate::features::portfolio::valuation::dto::PortfolioSummaryResponse;
use crate::http::errors::ApiError;
use crate::http::models::Data;
use axum::extract::State;
use axum::response::Json;

pub(crate) async fn portfolio_summary_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<Data<PortfolioSummaryResponse>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::portfolio_summary(&state, user).await?,
    }))
}

pub(crate) async fn portfolio_holdings_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<Data<Vec<HoldingResponse>>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::portfolio_holdings(&state, user).await?,
    }))
}
