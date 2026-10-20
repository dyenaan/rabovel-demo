use crate::app::state::GatewayState;
use crate::http::documentation::add_debug_routes;
use crate::http::ingress::ip_rate_limit_middleware;
use axum::extract::DefaultBodyLimit;
use axum::http::header;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::middleware;
use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;

/// Assemble the public HTTP surface from feature-owned route groups.
pub(crate) fn build(state: GatewayState) -> Router {
    let routes = crate::features::routes::feature_routes()
        .route("/health", get(crate::app::health::health_handler));
    // Sandbox/debug routes are unavailable behind the session gateway.
    let routes = if state.gateway_secret.is_none() {
        add_debug_routes(routes)
    } else {
        routes
    };
    routes
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    HeaderValue::from_str(&state.public_origin).expect("validated public origin"),
                )
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                    header::HeaderName::from_static("x-request-id"),
                ]),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            ip_rate_limit_middleware,
        ))
        .layer(middleware::from_fn(crate::http::observability::observe))
        .with_state(state)
}
