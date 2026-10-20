use crate::app::errors::ApplicationError;
use crate::app::errors::FailureKind;
use crate::features::catalog::instruments::Instrument;
use crate::features::catalog::instruments::InstrumentCatalog;
use identity::AuthError;

pub(crate) async fn instruments(state: &impl Ports) -> Result<InstrumentCatalog, ApplicationError> {
    let catalog = state
        .instrument_provider()
        .catalog()
        .await
        .map_err(|message| ApplicationError {
            kind: FailureKind::Unavailable,
            error: AuthError::PolicyViolation(message),
        })?;
    Ok(catalog)
}

pub(crate) fn catalog_unavailable(message: String) -> ApplicationError {
    ApplicationError {
        kind: FailureKind::Unavailable,
        error: AuthError::PolicyViolation(message),
    }
}

pub(crate) fn not_found(what: &str) -> ApplicationError {
    ApplicationError {
        kind: FailureKind::NotFound,
        error: AuthError::PolicyViolation(format!("{what} not found")),
    }
}

pub(crate) async fn catalog_instruments(
    state: &impl Ports,
) -> Result<Vec<Instrument>, ApplicationError> {
    Ok(state
        .instrument_provider()
        .catalog()
        .await
        .map_err(catalog_unavailable)?
        .instruments)
}

pub(crate) async fn instrument_by_market(
    state: &impl Ports,
    market_id: &str,
) -> Result<Instrument, ApplicationError> {
    catalog_instruments(state)
        .await?
        .into_iter()
        .find(|instrument| instrument.market_id.eq_ignore_ascii_case(market_id))
        .ok_or_else(|| not_found("market"))
}

pub(crate) async fn instrument_by_asset(
    state: &impl Ports,
    asset_id: &str,
) -> Result<Instrument, ApplicationError> {
    catalog_instruments(state)
        .await?
        .into_iter()
        .find(|instrument| instrument.asset_id.eq_ignore_ascii_case(asset_id))
        .ok_or_else(|| not_found("asset"))
}
pub(crate) trait Ports: crate::features::catalog::instruments::ports::CatalogSource {}
impl<T: crate::features::catalog::instruments::ports::CatalogSource + ?Sized> Ports for T {}
