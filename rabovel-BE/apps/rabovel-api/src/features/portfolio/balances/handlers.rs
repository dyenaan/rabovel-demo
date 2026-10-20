use crate::features::identity::access::actor::AuthenticatedUser;
use crate::http::errors::ApiError;
use axum::extract::State;
use axum::response::Json;
use portfolio_trading::PortfolioSnapshot;

pub(crate) async fn portfolio_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<PortfolioSnapshot>, ApiError> {
    Ok(Json(super::usecases::portfolio(&state, user).await?))
}
