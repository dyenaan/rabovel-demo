use crate::app::state::GatewayState;
use crate::app::state::GatewayStateBuilder;
use axum::Router;

pub fn router() -> Router {
    router_with_state(GatewayState::disabled())
}

/// Fail-closed router used when provider/bootstrap setup fails. It preserves
/// the configured browser origin so the frontend receives the actual auth
/// error instead of a misleading CORS failure.
pub fn fail_closed_router(public_origin: impl Into<String>) -> Router {
    router_with_state(
        GatewayStateBuilder::default()
            .public_origin(public_origin)
            .build(),
    )
}

pub(crate) fn router_with_state(state: GatewayState) -> Router {
    crate::http::router::build(state)
}
