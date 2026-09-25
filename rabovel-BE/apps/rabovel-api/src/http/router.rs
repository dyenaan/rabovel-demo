use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    middleware,
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

use super::features;
use crate::{add_debug_routes, health_handler, ip_rate_limit_middleware, GatewayState};

/// Assemble the public HTTP surface from feature-owned route groups.
pub(crate) fn build(state: GatewayState) -> Router {
    let routes = Router::new()
        .route("/health", get(health_handler))
        .merge(features::auth::routes())
        .merge(features::issuer::routes())
        .merge(features::onboarding::routes())
        .merge(features::wallets::routes())
        .merge(features::kyc::routes())
        .merge(features::portfolio::routes())
        .merge(features::tokenization::routes())
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    HeaderValue::from_str(&state.public_origin).expect("validated public origin"),
                )
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
                .allow_headers([
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                    header::HeaderName::from_static("x-request-id"),
                ]),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            ip_rate_limit_middleware,
        ));

    add_debug_routes(routes).with_state(state)
}
