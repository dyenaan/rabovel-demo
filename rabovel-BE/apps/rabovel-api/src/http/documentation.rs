//! Development-only documentation and sandbox routes.
#[cfg(debug_assertions)]
mod debug;
#[cfg(debug_assertions)]
pub(crate) use debug::add_debug_routes;

#[cfg(not(debug_assertions))]
pub(crate) fn add_debug_routes(
    router: axum::Router<crate::app::state::GatewayState>,
) -> axum::Router<crate::app::state::GatewayState> {
    router
}
