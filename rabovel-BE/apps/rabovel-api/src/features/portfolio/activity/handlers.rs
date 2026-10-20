use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::portfolio::activity::dto::ActivityEventResponse;
use crate::http::errors::ApiError;
use crate::http::models::Data;
use axum::extract::State;
use axum::response::Json;

pub(crate) async fn portfolio_activity_handler(
    State(state): State<super::context::FeatureContext>,
    user: AuthenticatedUser,
) -> Result<Json<Data<Vec<ActivityEventResponse>>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::portfolio_activity(&state, user).await?,
    }))
}
