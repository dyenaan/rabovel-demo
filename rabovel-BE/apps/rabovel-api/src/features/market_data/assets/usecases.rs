use crate::app::errors::ApplicationError;
use crate::features::catalog::instruments::usecases::catalog_instruments;
use crate::features::catalog::instruments::usecases::instrument_by_asset;
use crate::features::catalog::instruments::usecases::not_found;
use crate::features::catalog::instruments::Instrument;
use crate::features::catalog::instruments::InstrumentOrigin;
use crate::features::market_data::assets::dto::AssetResponse;
use crate::features::market_data::charts::calculations as market_views;
use crate::features::market_data::prices::usecases::reference_price;

pub(crate) async fn asset_response(
    state: &impl Ports,
    instrument: Instrument,
) -> Result<AssetResponse, ApplicationError> {
    let price = market_views::fixed2(
        reference_price(state, &instrument.ticker)?.into(),
        state.price_decimals(),
    );
    let (issuer_id, issuer_name, description, total_issued) = match instrument.origin {
        InstrumentOrigin::NgxListed => (
            "ngx".to_string(),
            "NGX (simulated listing)".to_string(),
            format!(
                "{} ({}): NGX-listed equity modeled as a minted, tradable token. Simulated; no underlying shares are held.",
                instrument.name, instrument.ticker
            ),
            None,
        ),
        InstrumentOrigin::Issuer => {
            let asset = state.assets()
                .get_asset_draft(&instrument.asset_id)
                .await
                .map_err(ApplicationError::from)?
                .ok_or_else(|| not_found("asset"))?;
            let issuer_name = state.organizations()
                .get_issuer_organization(&asset.issuer_user_id)
                .await
                .map_err(ApplicationError::from)?
                .map_or_else(|| "Demo issuer".to_string(), |org| org.legal_name);
            (
                asset.issuer_user_id,
                issuer_name,
                asset.draft.metadata.description,
                Some(asset.draft.authorized_units.to_string()),
            )
        }
    };
    Ok(AssetResponse {
        asset_id: instrument.asset_id,
        market_id: instrument.market_id,
        origin: instrument.origin,
        issuer_id,
        issuer_name,
        name: instrument.name,
        symbol: instrument.ticker,
        description,
        asset_class: "EQUITY".into(),
        settlement_currency: instrument.settlement_currency,
        nav: price.clone(),
        market_price: price,
        total_issued,
        lifecycle_status: "ACTIVE".into(),
        blockchain: "SOLANA".into(),
        token_standard: "SPL".into(),
        contract_address: instrument.mint_address,
    })
}

pub(crate) async fn list_assets(
    state: &impl Ports,
) -> Result<Vec<AssetResponse>, ApplicationError> {
    let mut assets = Vec::new();
    for instrument in catalog_instruments(state).await? {
        assets.push(asset_response(state, instrument).await?);
    }
    Ok(assets)
}

pub(crate) async fn get_asset(
    state: &impl Ports,
    asset_id: String,
) -> Result<AssetResponse, ApplicationError> {
    let instrument = instrument_by_asset(state, &asset_id).await?;
    asset_response(state, instrument).await
}
pub(crate) trait Ports:
    crate::features::issuer::assets::drafts::ports::AssetStore
    + crate::features::catalog::instruments::ports::CatalogSource
    + crate::features::issuer::organizations::ports::OrganizationStore
    + crate::features::market_data::prices::ports::PriceDecimals
    + crate::features::market_data::prices::ports::ReferencePrices
{
}
impl<
        T: crate::features::issuer::assets::drafts::ports::AssetStore
            + crate::features::catalog::instruments::ports::CatalogSource
            + crate::features::issuer::organizations::ports::OrganizationStore
            + crate::features::market_data::prices::ports::PriceDecimals
            + crate::features::market_data::prices::ports::ReferencePrices
            + ?Sized,
    > Ports for T
{
}
