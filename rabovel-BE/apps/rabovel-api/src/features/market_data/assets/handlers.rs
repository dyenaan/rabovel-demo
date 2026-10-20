use crate::features::market_data::assets::dto::AssetResponse;
use crate::http::errors::ApiError;
use crate::http::models::Data;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Json;

/// Every tradable instrument, NGX-listed or issuer-minted, as an asset.
pub(crate) async fn list_assets_handler(
    State(state): State<super::context::FeatureContext>,
) -> Result<Json<Data<Vec<AssetResponse>>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::list_assets(&state).await?,
    }))
}

pub(crate) async fn get_asset_handler(
    State(state): State<super::context::FeatureContext>,
    Path(asset_id): Path<String>,
) -> Result<Json<Data<AssetResponse>>, ApiError> {
    Ok(Json(crate::http::models::Data {
        data: super::usecases::get_asset(&state, asset_id).await?,
    }))
}
