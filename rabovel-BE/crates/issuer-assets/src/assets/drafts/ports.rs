use super::models::*;
use crate::assets::drafts::models::BackingEvidence;
use async_trait::async_trait;
use identity::AuthError;

#[async_trait]
pub trait AssetRepository: AssetQueries + Send + Sync {
    async fn create_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError>;

    async fn update_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError>;

    async fn delete_asset_draft(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
    ) -> Result<(), AuthError>;
    async fn approve_asset_for_demo_setup(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
    async fn set_asset_metadata_uri(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        metadata_uri: String,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
    async fn set_asset_image_uri(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        image_uri: String,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
    async fn list_asset_drafts(
        &self,
        issuer_user_id: &str,
    ) -> Result<Vec<AssetDraftRecord>, AuthError>;

    async fn save_asset_backing(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        backing: BackingEvidence,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
    async fn publish_asset_listing(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;

    async fn mark_asset_minted(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        mint_address: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
}

#[async_trait::async_trait]
pub trait AssetQueries: Send + Sync {
    async fn list_minted_assets(&self) -> Result<Vec<AssetDraftRecord>, AuthError>;

    async fn get_asset_draft(&self, asset_id: &str) -> Result<Option<AssetDraftRecord>, AuthError>;
}
