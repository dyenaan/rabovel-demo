use super::super::models::*;
use super::super::ports::AssetRepository;
use crate::assets::drafts::models::BackingEvidence;
use async_trait::async_trait;
use identity::AuthError;
use std::collections::HashMap;
use std::sync::RwLock;

fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("identity repository unavailable".into())
}
#[derive(Default)]
pub struct InMemoryAssets {
    asset_drafts: RwLock<HashMap<String, AssetDraftRecord>>,
}
#[async_trait]
impl AssetRepository for InMemoryAssets {
    async fn create_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError> {
        let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
        if assets.values().any(|existing| {
            existing.issuer_user_id == asset.issuer_user_id
                && (existing.draft.instrument_code == asset.draft.instrument_code
                    || existing.canonical_key == asset.canonical_key)
        }) {
            return Err(AuthError::PolicyViolation("an asset with this instrument code or market/ticker/share class already exists; edit the existing draft or use a distinct instrument identity".into()));
        }
        assets.insert(asset.asset_id.clone(), asset);
        Ok(())
    }
    async fn update_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError> {
        let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
        let existing = assets
            .get(&asset.asset_id)
            .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
        if existing.issuer_user_id != asset.issuer_user_id || existing.status != "draft" {
            return Err(AuthError::ForbiddenAction);
        }
        if assets.values().any(|other| {
            other.asset_id != asset.asset_id
                && other.issuer_user_id == asset.issuer_user_id
                && (other.draft.instrument_code == asset.draft.instrument_code
                    || other.canonical_key == asset.canonical_key)
        }) {
            return Err(AuthError::PolicyViolation("an asset with this instrument code or market/ticker/share class already exists; edit the existing draft or use a distinct instrument identity".into()));
        }
        assets.insert(asset.asset_id.clone(), asset);
        Ok(())
    }
    async fn delete_asset_draft(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
    ) -> Result<(), AuthError> {
        let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
        let existing = assets
            .get(asset_id)
            .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
        if existing.issuer_user_id != issuer_user_id || existing.status != "draft" {
            return Err(AuthError::ForbiddenAction);
        }
        assets.remove(asset_id);
        Ok(())
    }
    async fn approve_asset_for_demo_setup(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
        let asset = assets
            .get_mut(asset_id)
            .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
        if asset.issuer_user_id != issuer_user_id {
            return Err(AuthError::ForbiddenAction);
        }
        match asset.status.as_str() {
            "draft" => {
                asset.status = "approved_for_setup".into();
                asset.updated_at = updated_at;
            }
            "approved_for_setup" => {}
            _ => {
                return Err(AuthError::PolicyViolation(
                    "asset cannot be submitted from its current status".into(),
                ))
            }
        }
        Ok(asset.clone())
    }
    async fn set_asset_metadata_uri(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        metadata_uri: String,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
        let asset = assets
            .get_mut(asset_id)
            .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
        if asset.issuer_user_id != issuer_user_id {
            return Err(AuthError::ForbiddenAction);
        }
        asset.draft.metadata.metadata_uri = Some(metadata_uri);
        asset.updated_at = updated_at;
        Ok(asset.clone())
    }
    async fn set_asset_image_uri(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        image_uri: String,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
        let asset = assets
            .get_mut(asset_id)
            .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
        if asset.issuer_user_id != issuer_user_id || asset.status != "draft" {
            return Err(AuthError::ForbiddenAction);
        }
        asset.draft.metadata.image_uri = Some(image_uri);
        asset.updated_at = updated_at;
        Ok(asset.clone())
    }
    async fn list_asset_drafts(
        &self,
        issuer_user_id: &str,
    ) -> Result<Vec<AssetDraftRecord>, AuthError> {
        let mut assets: Vec<_> = self
            .asset_drafts
            .read()
            .map_err(|_| state_unavailable())?
            .values()
            .filter(|asset| asset.issuer_user_id == issuer_user_id)
            .cloned()
            .collect();
        assets.sort_by_key(|asset| asset.created_at);
        Ok(assets)
    }

    async fn save_asset_backing(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        backing: BackingEvidence,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
        let asset = assets
            .get_mut(asset_id)
            .ok_or_else(|| AuthError::PolicyViolation("asset not found".into()))?;
        if asset.issuer_user_id != issuer_user_id || asset.status != "minted" {
            return Err(AuthError::ForbiddenAction);
        }
        asset.draft.backing = Some(backing);
        asset.updated_at = updated_at;
        Ok(asset.clone())
    }
    async fn publish_asset_listing(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
        let asset = assets
            .get_mut(asset_id)
            .ok_or_else(|| AuthError::PolicyViolation("asset not found".into()))?;
        if asset.issuer_user_id != issuer_user_id
            || asset.status != "minted"
            || asset
                .draft
                .backing
                .as_ref()
                .is_none_or(|backing| backing.verification_status != "verified")
        {
            return Err(AuthError::ForbiddenAction);
        }
        asset.draft.listing_status = "live".into();
        asset.updated_at = updated_at;
        Ok(asset.clone())
    }

    async fn mark_asset_minted(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        mint_address: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
        let asset = assets
            .get_mut(asset_id)
            .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
        if asset.issuer_user_id != issuer_user_id
            || !matches!(asset.status.as_str(), "approved_for_setup" | "minted")
        {
            return Err(AuthError::ForbiddenAction);
        }
        if asset
            .mint_address
            .as_deref()
            .is_some_and(|existing| existing != mint_address)
        {
            return Err(AuthError::PolicyViolation(
                "asset already references a different mint".into(),
            ));
        }
        asset.status = "minted".into();
        asset.mint_address = Some(mint_address.into());
        asset.updated_at = updated_at;
        Ok(asset.clone())
    }
}
#[async_trait::async_trait]
impl super::super::ports::AssetQueries for InMemoryAssets {
    async fn list_minted_assets(&self) -> Result<Vec<AssetDraftRecord>, AuthError> {
        let mut assets: Vec<_> = self
            .asset_drafts
            .read()
            .map_err(|_| state_unavailable())?
            .values()
            .filter(|asset| asset.status == "minted")
            .cloned()
            .collect();
        assets.sort_by_key(|asset| asset.created_at);
        Ok(assets)
    }
    async fn get_asset_draft(&self, asset_id: &str) -> Result<Option<AssetDraftRecord>, AuthError> {
        Ok(self
            .asset_drafts
            .read()
            .map_err(|_| state_unavailable())?
            .get(asset_id)
            .cloned())
    }
}
