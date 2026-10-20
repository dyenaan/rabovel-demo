use crate::app::state::GatewayState;
use axum::extract::FromRef;
use std::sync::Arc;
#[derive(Clone)]
pub(crate) struct FeatureContext {
    instrument_provider: Arc<dyn crate::features::catalog::instruments::ports::InstrumentProvider>,
}
impl FromRef<GatewayState> for FeatureContext {
    fn from_ref(state: &GatewayState) -> Self {
        Self {
            instrument_provider: state.instrument_provider.clone(),
        }
    }
}
impl crate::features::catalog::instruments::ports::CatalogSource for FeatureContext {
    fn instrument_provider(
        &self,
    ) -> &Arc<dyn crate::features::catalog::instruments::ports::InstrumentProvider> {
        &self.instrument_provider
    }
}
