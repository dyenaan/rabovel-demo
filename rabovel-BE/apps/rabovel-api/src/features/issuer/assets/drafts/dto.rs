use issuer_assets::assets::drafts::models::AssetMetadata;
use issuer_assets::assets::drafts::models::NewAssetDraft;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateAssetDraftRequest {
    pub instrument_code: String,
    pub name: String,
    pub ticker: String,
    pub market: String,
    pub share_class: String,
    pub asset_type: String,
    pub decimals: u8,
    /// Integer base units, encoded as a string to preserve exact values in browsers.
    pub authorized_units: String,
    pub settlement_currency: String,
    pub representation: String,
    pub rights_description: String,
    pub disclosure: String,
    pub metadata: AssetMetadata,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AssetDraftResponse {
    pub asset_id: String,
    pub issuer_user_id: String,
    pub status: String,
    pub issued_units: String,
    pub mint_address: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub draft: NewAssetDraft,
}
