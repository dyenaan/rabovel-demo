use super::*;
use issuer_assets::assets::drafts::models::is_valid_ticker;
use issuer_assets::assets::drafts::models::AssetDraftRecord;
use issuer_assets::assets::drafts::models::AssetMetadata;
use issuer_assets::assets::drafts::models::NewAssetDraft;
use issuer_assets::assets::drafts::models::DEMO_DISCLOSURE;
use issuer_assets::assets::drafts::ports::AssetRepository;
use issuer_assets::assets::drafts::repositories::memory::InMemoryAssets;
use std::sync::Arc;

fn issuer_asset(asset_id: &str, ticker: &str, status: &str, listing: &str) -> AssetDraftRecord {
    AssetDraftRecord {
        asset_id: asset_id.into(),
        issuer_user_id: "usr_issuer".into(),
        canonical_key: format!("NGX:{ticker}:ORDINARY"),
        draft: NewAssetDraft {
            instrument_code: format!("{ticker}-CODE"),
            name: format!("{ticker} Demo"),
            ticker: ticker.into(),
            market: "NGX".into(),
            share_class: "Ordinary".into(),
            asset_type: "equity".into(),
            decimals: 0,
            authorized_units: 1_000,
            settlement_currency: "CNGN".into(),
            representation: "1 token = 1 simulated share".into(),
            rights_description: "Simulated".into(),
            disclosure: DEMO_DISCLOSURE.into(),
            metadata: AssetMetadata {
                name: format!("{ticker} Demo"),
                symbol: ticker.into(),
                description: "Demo".into(),
                image_uri: None,
                external_url: None,
                metadata_uri: None,
                additional_metadata: vec![],
            },
            backing: None,
            listing_status: listing.into(),
        },
        status: status.into(),
        created_at: 1,
        updated_at: 1,
        mint_address: Some(format!("mint-{asset_id}")),
    }
}

#[tokio::test]
async fn ngx_catalog_matches_the_watchlist_format() {
    let catalog = SimulatedInstrumentProvider.catalog().await.unwrap();
    assert_eq!(catalog.provider, "rabovel_simulated_instrument_provider");
    assert!(catalog.simulated);
    assert_eq!(catalog.instruments.len(), 146);
    assert!(catalog.instruments.iter().all(|instrument| {
        is_valid_ticker(&instrument.ticker)
            && instrument.market_id == format!("mkt_ngx_{}", instrument.ticker.to_lowercase())
            && instrument.origin == InstrumentOrigin::NgxListed
            && instrument.asset_id == format!("ast_ngx_{}", instrument.ticker.to_lowercase())
            && instrument.settlement_currency == "CNGN"
    }));
}

#[tokio::test]
async fn live_issuer_assets_join_the_catalog_automatically() {
    let repository = Arc::new(InMemoryAssets::default());
    for asset in [
        issuer_asset("ast_live", "RABOTELCO", "minted", "live"),
        issuer_asset("ast_unlisted", "RABOBANK", "minted", "not_listed"),
        issuer_asset("ast_legacy", "RABO-NG-TELCO", "minted", "live"),
    ] {
        repository.create_asset_draft(asset).await.unwrap();
    }
    let catalog = UnifiedInstrumentCatalog::new(repository);

    let instruments = catalog.catalog().await.unwrap().instruments;
    assert_eq!(instruments.len(), 147);
    let live = catalog.find("rabotelco").await.unwrap().unwrap();
    assert_eq!(live.origin, InstrumentOrigin::Issuer);
    assert_eq!(live.market_id, "mkt_ngx_rabotelco");
    assert_eq!(live.asset_id, "ast_live");
    assert_eq!(live.mint_address.as_deref(), Some("mint-ast_live"));
    assert!(catalog.find("RABOBANK").await.unwrap().is_none());
    assert!(catalog.find("RABO-NG-TELCO").await.unwrap().is_none());
    assert_eq!(
        catalog.find("gtco").await.unwrap().unwrap().origin,
        InstrumentOrigin::NgxListed
    );
}
