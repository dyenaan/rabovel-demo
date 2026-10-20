use super::super::models::*;
use super::super::ports::AssetQueries;
use super::super::ports::AssetRepository;
use async_trait::async_trait;
use identity::AuthError;
use sqlx::PgPool;

pub struct PgAssets {
    pool: PgPool,
}
impl PgAssets {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
fn db_error(_context: &str) -> impl Fn(sqlx::Error) -> AuthError + '_ {
    |_| AuthError::ServiceUnavailable
}

fn to_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}
fn to_u64(value: i64) -> u64 {
    value.max(0) as u64
}
fn asset_row(
    row: (
        String,
        String,
        String,
        serde_json::Value,
        String,
        i64,
        i64,
        Option<String>,
    ),
) -> Result<AssetDraftRecord, AuthError> {
    let (
        asset_id,
        issuer_user_id,
        canonical_key,
        draft,
        status,
        created_at,
        updated_at,
        mint_address,
    ) = row;
    let draft = serde_json::from_value(draft)
        .map_err(|_| AuthError::PolicyViolation("invalid stored asset draft".into()))?;
    Ok(AssetDraftRecord {
        asset_id,
        issuer_user_id,
        canonical_key,
        draft,
        status,
        created_at: to_u64(created_at),
        updated_at: to_u64(updated_at),
        mint_address,
    })
}
#[async_trait]
impl AssetRepository for PgAssets {
    async fn create_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError> {
        let draft = serde_json::to_value(&asset.draft)
            .map_err(|_| AuthError::PolicyViolation("failed to encode asset draft".into()))?;
        sqlx::query("INSERT INTO asset_drafts (asset_id, issuer_user_id, canonical_key, instrument_code, draft, status, created_at, updated_at, mint_address) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)")
            .bind(&asset.asset_id).bind(&asset.issuer_user_id).bind(&asset.canonical_key)
            .bind(&asset.draft.instrument_code).bind(draft).bind(&asset.status)
            .bind(to_i64(asset.created_at)).bind(to_i64(asset.updated_at)).bind(&asset.mint_address)
            .execute(&self.pool).await.map_err(|error| match &error {
                sqlx::Error::Database(db) if db.is_unique_violation() => AuthError::PolicyViolation("an asset with this instrument code or market/ticker/share class already exists; edit the existing draft or use a distinct instrument identity".into()),
                _ => db_error("failed to create asset draft")(error),
            })?;
        Ok(())
    }
    async fn update_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError> {
        let draft = serde_json::to_value(&asset.draft)
            .map_err(|_| AuthError::PolicyViolation("failed to encode asset draft".into()))?;
        let result = sqlx::query("UPDATE asset_drafts SET canonical_key = $1, instrument_code = $2, draft = $3, updated_at = $4 WHERE asset_id = $5 AND issuer_user_id = $6 AND status = 'draft'")
            .bind(&asset.canonical_key).bind(&asset.draft.instrument_code).bind(draft)
            .bind(to_i64(asset.updated_at)).bind(&asset.asset_id).bind(&asset.issuer_user_id)
            .execute(&self.pool).await.map_err(|error| match &error {
                sqlx::Error::Database(db) if db.is_unique_violation() => AuthError::PolicyViolation("an asset with this instrument code or market/ticker/share class already exists; edit the existing draft or use a distinct instrument identity".into()),
                _ => db_error("failed to update asset draft")(error),
            })?;
        if result.rows_affected() != 1 {
            return Err(AuthError::ForbiddenAction);
        }
        Ok(())
    }
    async fn delete_asset_draft(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
    ) -> Result<(), AuthError> {
        let result = sqlx::query(
            "DELETE FROM asset_drafts WHERE asset_id = $1 AND issuer_user_id = $2 AND status = 'draft'",
        )
        .bind(asset_id)
        .bind(issuer_user_id)
        .execute(&self.pool)
        .await
        .map_err(db_error("failed to delete asset draft"))?;
        if result.rows_affected() != 1 {
            return Err(AuthError::ForbiddenAction);
        }
        Ok(())
    }
    async fn approve_asset_for_demo_setup(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let row: Option<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("UPDATE asset_drafts SET status = 'approved_for_setup', updated_at = $1 WHERE asset_id = $2 AND issuer_user_id = $3 AND status = 'draft' RETURNING asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address")
            .bind(to_i64(updated_at)).bind(asset_id).bind(issuer_user_id).fetch_optional(&self.pool).await.map_err(db_error("failed to approve demo asset"))?;
        if let Some(row) = row {
            return asset_row(row);
        }
        let existing = self
            .get_asset_draft(asset_id)
            .await?
            .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
        if existing.issuer_user_id != issuer_user_id {
            return Err(AuthError::ForbiddenAction);
        }
        if existing.status == "approved_for_setup" {
            return Ok(existing);
        }
        Err(AuthError::PolicyViolation(
            "asset cannot be submitted from its current status".into(),
        ))
    }
    async fn set_asset_metadata_uri(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        metadata_uri: String,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let row: Option<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("UPDATE asset_drafts SET draft = jsonb_set(draft, '{metadata,metadata_uri}', to_jsonb($1::text), true), updated_at = $2 WHERE asset_id = $3 AND issuer_user_id = $4 RETURNING asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address")
            .bind(metadata_uri).bind(to_i64(updated_at)).bind(asset_id).bind(issuer_user_id)
            .fetch_optional(&self.pool).await.map_err(db_error("failed to save asset metadata uri"))?;
        row.map(asset_row)
            .transpose()?
            .ok_or(AuthError::ForbiddenAction)
    }
    async fn set_asset_image_uri(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        image_uri: String,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let row: Option<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("UPDATE asset_drafts SET draft = jsonb_set(draft, '{metadata,image_uri}', to_jsonb($1::text), true), updated_at = $2 WHERE asset_id = $3 AND issuer_user_id = $4 AND status = 'draft' RETURNING asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address")
            .bind(image_uri).bind(to_i64(updated_at)).bind(asset_id).bind(issuer_user_id).fetch_optional(&self.pool).await.map_err(db_error("failed to save asset image uri"))?;
        row.map(asset_row)
            .transpose()?
            .ok_or(AuthError::ForbiddenAction)
    }
    async fn list_asset_drafts(
        &self,
        issuer_user_id: &str,
    ) -> Result<Vec<AssetDraftRecord>, AuthError> {
        let rows: Vec<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("SELECT asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address FROM asset_drafts WHERE issuer_user_id = $1 ORDER BY created_at")
            .bind(issuer_user_id).fetch_all(&self.pool).await.map_err(db_error("failed to list asset drafts"))?;
        rows.into_iter().map(asset_row).collect()
    }

    async fn save_asset_backing(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        backing: BackingEvidence,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let backing = serde_json::to_value(backing)
            .map_err(|_| AuthError::PolicyViolation("failed to encode backing evidence".into()))?;
        let row: Option<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("UPDATE asset_drafts SET draft = jsonb_set(draft, '{backing}', $1, true), updated_at = $2 WHERE asset_id = $3 AND issuer_user_id = $4 AND status = 'minted' RETURNING asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address")
            .bind(backing).bind(to_i64(updated_at)).bind(asset_id).bind(issuer_user_id)
            .fetch_optional(&self.pool).await.map_err(db_error("failed to save backing evidence"))?;
        row.map(asset_row)
            .transpose()?
            .ok_or(AuthError::ForbiddenAction)
    }
    async fn publish_asset_listing(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let row: Option<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("UPDATE asset_drafts SET draft = jsonb_set(draft, '{listing_status}', to_jsonb('live'::text), true), updated_at = $1 WHERE asset_id = $2 AND issuer_user_id = $3 AND status = 'minted' AND draft #>> '{backing,verification_status}' = 'verified' RETURNING asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address")
            .bind(to_i64(updated_at)).bind(asset_id).bind(issuer_user_id)
            .fetch_optional(&self.pool).await.map_err(db_error("failed to publish asset listing"))?;
        row.map(asset_row)
            .transpose()?
            .ok_or(AuthError::ForbiddenAction)
    }

    async fn mark_asset_minted(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        mint_address: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError> {
        let row: Option<(String,String,String,serde_json::Value,String,i64,i64,Option<String>)> = sqlx::query_as("UPDATE asset_drafts SET status='minted', mint_address=$1, updated_at=$2 WHERE asset_id=$3 AND issuer_user_id=$4 AND status IN ('approved_for_setup','minted') AND (mint_address IS NULL OR mint_address=$1) RETURNING asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address")
            .bind(mint_address).bind(to_i64(updated_at)).bind(asset_id).bind(issuer_user_id).fetch_optional(&self.pool).await.map_err(db_error("failed to mark asset minted"))?;
        row.map(asset_row)
            .transpose()?
            .ok_or(AuthError::ForbiddenAction)
    }
}
#[async_trait::async_trait]
impl super::super::ports::AssetQueries for PgAssets {
    async fn list_minted_assets(&self) -> Result<Vec<AssetDraftRecord>, AuthError> {
        let rows: Vec<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("SELECT asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address FROM asset_drafts WHERE status = 'minted' ORDER BY created_at")
            .fetch_all(&self.pool).await.map_err(db_error("failed to list minted assets"))?;
        rows.into_iter().map(asset_row).collect()
    }
    async fn get_asset_draft(&self, asset_id: &str) -> Result<Option<AssetDraftRecord>, AuthError> {
        let row: Option<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("SELECT asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address FROM asset_drafts WHERE asset_id = $1")
            .bind(asset_id).fetch_optional(&self.pool).await.map_err(db_error("failed to load asset draft"))?;
        row.map(asset_row).transpose()
    }
}
