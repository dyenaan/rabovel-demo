//! Postgres-backed [`Repository`] implementation.
//!
//! Uses runtime `query`/`query_as` calls with `#[derive(sqlx::FromRow)]`
//! structs rather than the compile-time-checked `query!`/`query_as!` macros
//! -- slightly less compile-time safety, but it means building this crate
//! never requires a live database connection (the macros need either a live
//! DB or a checked-in `.sqlx` offline cache at compile time).

use async_trait::async_trait;
use domain::auth::{
    AuthError, AuthPrincipal, AuthProvider, IdentityProfile, RiskTier, SupportedChain, UserRole,
    WalletAction, WalletConnection, WalletProvider,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashSet;

use identity::{
    AssetDraftRecord, AssetSetupOperation, IssuerOrganization, KycCaseRecord, KycCaseStatus,
    PasswordCredential, Repository, UserRecord,
};

pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub async fn connect(database_url: &str) -> Result<Self, String> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| format!("failed to connect to Postgres: {e}"))?;
        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), String> {
        sqlx::migrate!("../../migrations")
            .run(&self.pool)
            .await
            .map_err(|e| format!("failed to run database migrations: {e}"))
    }
}

fn db_error(context: &str) -> impl Fn(sqlx::Error) -> AuthError + '_ {
    move |e| AuthError::PolicyViolation(format!("{context}: {e}"))
}

/// Reuses `T`'s own `serde` impl to encode/decode the enum column as the
/// same string domain's serde `rename_all = "snake_case"` attributes
/// already produce, rather than hand-duplicating each variant's string form
/// (and risking the two falling out of sync).
fn enum_to_column<T: serde::Serialize>(value: &T) -> Result<String, AuthError> {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .ok_or_else(|| AuthError::PolicyViolation("failed to encode enum column".to_string()))
}

fn column_to_enum<T: serde::de::DeserializeOwned>(value: &str) -> Result<T, AuthError> {
    serde_json::from_value(serde_json::Value::String(value.to_string()))
        .map_err(|_| AuthError::PolicyViolation(format!("invalid stored enum value: {value}")))
}

fn to_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}

fn to_u64(value: i64) -> u64 {
    value.max(0) as u64
}

#[derive(sqlx::FromRow)]
struct UserRow {
    user_id: String,
    email: String,
    email_verified: bool,
    phone_verified: bool,
    kyc_verified: bool,
    terms_version: Option<String>,
    mfa_enrolled: bool,
    trusted_device: bool,
    risk_tier: String,
    compliance_checked_at: Option<i64>,
    auth_provider: String,
    roles: Vec<String>,
    permissions: Vec<String>,
}

impl UserRow {
    fn into_record(self, wallet_ids: Vec<String>) -> Result<UserRecord, AuthError> {
        let risk_tier: RiskTier = column_to_enum(&self.risk_tier)?;
        let auth_provider: AuthProvider = column_to_enum(&self.auth_provider)?;
        let roles: HashSet<UserRole> = self
            .roles
            .iter()
            .map(|r| column_to_enum(r))
            .collect::<Result<_, _>>()?;
        let permissions: HashSet<WalletAction> = self
            .permissions
            .iter()
            .map(|p| column_to_enum(p))
            .collect::<Result<_, _>>()?;
        let profile = IdentityProfile::reconstitute(
            self.user_id.clone(),
            self.email,
            self.email_verified,
            self.phone_verified,
            self.kyc_verified,
            self.terms_version,
            self.mfa_enrolled,
            self.trusted_device,
            risk_tier,
            self.compliance_checked_at.map(to_u64),
            wallet_ids,
            auth_provider,
        );
        let principal = AuthPrincipal::new(self.user_id, roles, permissions);
        Ok(UserRecord { profile, principal })
    }
}

#[derive(sqlx::FromRow)]
struct WalletRow {
    wallet_id: String,
    owner_user_id: String,
    chain: String,
    address: String,
    provider: String,
    verified_at: i64,
}

impl WalletRow {
    fn into_wallet_connection(self) -> Result<WalletConnection, AuthError> {
        let chain: SupportedChain = column_to_enum(&self.chain)?;
        let provider: WalletProvider = column_to_enum(&self.provider)?;
        WalletConnection::from_verified_proof(
            self.wallet_id,
            self.owner_user_id,
            chain,
            self.address,
            provider,
            to_u64(self.verified_at),
        )
    }
}

#[derive(sqlx::FromRow)]
struct KycCaseRow {
    case_id: String,
    user_id: String,
    status: String,
    risk_tier: Option<String>,
    submitted_at: i64,
    finalized_at: Option<i64>,
}

impl KycCaseRow {
    fn into_record(self) -> Result<KycCaseRecord, AuthError> {
        let status = self.status.parse::<KycCaseStatus>().map_err(|()| {
            AuthError::PolicyViolation(format!("invalid stored kyc status: {}", self.status))
        })?;
        let risk_tier = self.risk_tier.as_deref().map(column_to_enum).transpose()?;
        Ok(KycCaseRecord {
            case_id: self.case_id,
            user_id: self.user_id,
            status,
            risk_tier,
            submitted_at: to_u64(self.submitted_at),
            finalized_at: self.finalized_at.map(to_u64),
        })
    }
}

impl PgRepository {
    async fn wallet_ids_for_user(&self, user_id: &str) -> Result<Vec<String>, AuthError> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT wallet_id FROM wallets WHERE owner_user_id = $1")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await
                .map_err(db_error("failed to load wallet ids"))?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    async fn load_user(&self, user_id: &str) -> Result<Option<UserRecord>, AuthError> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT user_id, email, email_verified, phone_verified, kyc_verified, terms_version, \
             mfa_enrolled, trusted_device, risk_tier, compliance_checked_at, auth_provider, roles, permissions \
             FROM users WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error("failed to load user"))?;
        match row {
            Some(row) => {
                let wallet_ids = self.wallet_ids_for_user(user_id).await?;
                Ok(Some(row.into_record(wallet_ids)?))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl Repository for PgRepository {
    async fn create_password_user(
        &self,
        credential: PasswordCredential,
        profile: IdentityProfile,
        principal: AuthPrincipal,
    ) -> Result<UserRecord, AuthError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(db_error("failed to start transaction"))?;
        let roles: Vec<String> = principal
            .roles_iter()
            .map(|role| enum_to_column(&role))
            .collect::<Result<_, _>>()?;
        let permissions: Vec<String> = principal
            .permissions_iter()
            .map(|permission| enum_to_column(&permission))
            .collect::<Result<_, _>>()?;
        sqlx::query(
            "INSERT INTO users (user_id, email, email_verified, phone_verified, kyc_verified, terms_version, mfa_enrolled, trusted_device, risk_tier, compliance_checked_at, auth_provider, roles, permissions) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
        )
        .bind(&credential.user_id).bind(profile.email()).bind(profile.email_verified())
        .bind(profile.phone_verified()).bind(profile.kyc_verified()).bind(profile.terms_version())
        .bind(profile.mfa_enrolled()).bind(profile.trusted_device()).bind(enum_to_column(&profile.risk_tier())?)
        .bind(profile.compliance_checked_at().map(to_i64)).bind(enum_to_column(&profile.auth_provider())?)
        .bind(&roles).bind(&permissions).execute(&mut *tx).await.map_err(db_error("failed to insert password user"))?;
        sqlx::query("INSERT INTO password_credentials (user_id, email, display_name, password_hash) VALUES ($1,$2,$3,$4)")
            .bind(&credential.user_id).bind(&credential.email).bind(&credential.display_name).bind(&credential.password_hash)
            .execute(&mut *tx).await.map_err(|error| match &error {
                sqlx::Error::Database(db) if db.is_unique_violation() => AuthError::PolicyViolation("an account already exists for this email".to_string()),
                _ => db_error("failed to insert password credential")(error),
            })?;
        tx.commit()
            .await
            .map_err(db_error("failed to commit password user"))?;
        Ok(UserRecord { profile, principal })
    }

    async fn get_password_credential(
        &self,
        email: &str,
    ) -> Result<Option<PasswordCredential>, AuthError> {
        let row: Option<(String, String, String, String)> = sqlx::query_as("SELECT user_id, email, display_name, password_hash FROM password_credentials WHERE email = $1")
            .bind(email).fetch_optional(&self.pool).await.map_err(db_error("failed to load password credential"))?;
        Ok(row.map(
            |(user_id, email, display_name, password_hash)| PasswordCredential {
                user_id,
                email,
                display_name,
                password_hash,
            },
        ))
    }

    async fn get_password_credential_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Option<PasswordCredential>, AuthError> {
        let row: Option<(String, String, String, String)> = sqlx::query_as("SELECT user_id, email, display_name, password_hash FROM password_credentials WHERE user_id = $1")
            .bind(user_id).fetch_optional(&self.pool).await.map_err(db_error("failed to load password credential"))?;
        Ok(row.map(
            |(user_id, email, display_name, password_hash)| PasswordCredential {
                user_id,
                email,
                display_name,
                password_hash,
            },
        ))
    }

    async fn save_issuer_organization(
        &self,
        organization: IssuerOrganization,
    ) -> Result<(), AuthError> {
        sqlx::query(
            "INSERT INTO issuer_organizations (user_id, legal_name, organization_type, registration_number, jurisdiction, registered_address, representative_name, representative_title, document_reference, approved_at) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) \
             ON CONFLICT (user_id) DO UPDATE SET legal_name = EXCLUDED.legal_name, organization_type = EXCLUDED.organization_type, registration_number = EXCLUDED.registration_number, jurisdiction = EXCLUDED.jurisdiction, registered_address = EXCLUDED.registered_address, representative_name = EXCLUDED.representative_name, representative_title = EXCLUDED.representative_title, document_reference = EXCLUDED.document_reference, approved_at = EXCLUDED.approved_at, updated_at = now()",
        )
        .bind(&organization.user_id)
        .bind(&organization.legal_name)
        .bind(&organization.organization_type)
        .bind(&organization.registration_number)
        .bind(&organization.jurisdiction)
        .bind(&organization.registered_address)
        .bind(&organization.representative_name)
        .bind(&organization.representative_title)
        .bind(&organization.document_reference)
        .bind(to_i64(organization.approved_at))
        .execute(&self.pool)
        .await
        .map_err(db_error("failed to save issuer organization"))?;
        Ok(())
    }

    async fn get_issuer_organization(
        &self,
        user_id: &str,
    ) -> Result<Option<IssuerOrganization>, AuthError> {
        let row: Option<(String, String, String, String, String, String, String, String, String, i64)> = sqlx::query_as(
            "SELECT user_id, legal_name, organization_type, registration_number, jurisdiction, registered_address, representative_name, representative_title, document_reference, approved_at FROM issuer_organizations WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error("failed to load issuer organization"))?;
        Ok(row.map(
            |(
                user_id,
                legal_name,
                organization_type,
                registration_number,
                jurisdiction,
                registered_address,
                representative_name,
                representative_title,
                document_reference,
                approved_at,
            )| IssuerOrganization {
                user_id,
                legal_name,
                organization_type,
                registration_number,
                jurisdiction,
                registered_address,
                representative_name,
                representative_title,
                document_reference,
                approved_at: to_u64(approved_at),
            },
        ))
    }

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

    async fn list_minted_assets(&self) -> Result<Vec<AssetDraftRecord>, AuthError> {
        let rows: Vec<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("SELECT asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address FROM asset_drafts WHERE status = 'minted' ORDER BY created_at")
            .fetch_all(&self.pool).await.map_err(db_error("failed to list minted assets"))?;
        rows.into_iter().map(asset_row).collect()
    }

    async fn save_asset_backing(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        backing: domain::assets::BackingEvidence,
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

    async fn get_asset_draft(&self, asset_id: &str) -> Result<Option<AssetDraftRecord>, AuthError> {
        let row: Option<(String, String, String, serde_json::Value, String, i64, i64, Option<String>)> = sqlx::query_as("SELECT asset_id, issuer_user_id, canonical_key, draft, status, created_at, updated_at, mint_address FROM asset_drafts WHERE asset_id = $1")
            .bind(asset_id).fetch_optional(&self.pool).await.map_err(db_error("failed to load asset draft"))?;
        row.map(asset_row).transpose()
    }

    async fn create_asset_setup_operation(
        &self,
        operation: AssetSetupOperation,
    ) -> Result<AssetSetupOperation, AuthError> {
        let stages = serde_json::to_value(&operation.stages)
            .map_err(|_| AuthError::PolicyViolation("failed to encode setup stages".into()))?;
        sqlx::query("INSERT INTO asset_setup_operations (operation_id, asset_id, issuer_user_id, network, mint_address, status, stages, mint_config_address, allow_list_address, block_list_address, thaw_extra_metas_address, error, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14) ON CONFLICT (asset_id) DO NOTHING")
            .bind(&operation.operation_id).bind(&operation.asset_id).bind(&operation.issuer_user_id).bind(&operation.network)
            .bind(&operation.mint_address).bind(&operation.status).bind(stages).bind(&operation.mint_config_address)
            .bind(&operation.allow_list_address).bind(&operation.block_list_address).bind(&operation.thaw_extra_metas_address)
            .bind(&operation.error).bind(to_i64(operation.created_at)).bind(to_i64(operation.updated_at))
            .execute(&self.pool).await.map_err(db_error("failed to create asset setup operation"))?;
        self.get_asset_setup_operation(&operation.asset_id)
            .await?
            .ok_or_else(|| AuthError::PolicyViolation("asset setup operation unavailable".into()))
    }

    async fn save_asset_setup_operation(
        &self,
        operation: AssetSetupOperation,
    ) -> Result<(), AuthError> {
        let stages = serde_json::to_value(&operation.stages)
            .map_err(|_| AuthError::PolicyViolation("failed to encode setup stages".into()))?;
        let result = sqlx::query("UPDATE asset_setup_operations SET status=$1, stages=$2, mint_config_address=$3, allow_list_address=$4, block_list_address=$5, thaw_extra_metas_address=$6, error=$7, updated_at=$8 WHERE operation_id=$9 AND asset_id=$10")
            .bind(&operation.status).bind(stages).bind(&operation.mint_config_address).bind(&operation.allow_list_address)
            .bind(&operation.block_list_address).bind(&operation.thaw_extra_metas_address).bind(&operation.error)
            .bind(to_i64(operation.updated_at)).bind(&operation.operation_id).bind(&operation.asset_id)
            .execute(&self.pool).await.map_err(db_error("failed to save asset setup operation"))?;
        if result.rows_affected() != 1 {
            return Err(AuthError::PolicyViolation(
                "asset setup operation not found".into(),
            ));
        }
        Ok(())
    }

    async fn get_asset_setup_operation(
        &self,
        asset_id: &str,
    ) -> Result<Option<AssetSetupOperation>, AuthError> {
        let row: Option<(String,String,String,String,String,String,serde_json::Value,Option<String>,Option<String>,Option<String>,Option<String>,Option<String>,i64,i64)> = sqlx::query_as("SELECT operation_id, asset_id, issuer_user_id, network, mint_address, status, stages, mint_config_address, allow_list_address, block_list_address, thaw_extra_metas_address, error, created_at, updated_at FROM asset_setup_operations WHERE asset_id=$1")
            .bind(asset_id).fetch_optional(&self.pool).await.map_err(db_error("failed to load asset setup operation"))?;
        row.map(setup_operation_row).transpose()
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

    async fn get_or_create_user_by_identity(
        &self,
        issuer: &str,
        subject: &str,
        new_user_id: &str,
        new_profile: IdentityProfile,
        new_principal: AuthPrincipal,
    ) -> Result<(String, UserRecord), AuthError> {
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT user_id FROM identity_links WHERE issuer = $1 AND subject = $2")
                .bind(issuer)
                .bind(subject)
                .fetch_optional(&self.pool)
                .await
                .map_err(db_error("failed to look up identity link"))?;

        if let Some((user_id,)) = existing {
            let record = self.load_user(&user_id).await?.ok_or_else(|| {
                AuthError::PolicyViolation("identity link with no user row".to_string())
            })?;
            return Ok((user_id, record));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(db_error("failed to start transaction"))?;

        let roles: Vec<String> = new_principal
            .roles_iter()
            .map(|r| enum_to_column(&r))
            .collect::<Result<_, _>>()?;
        let permissions: Vec<String> = new_principal
            .permissions_iter()
            .map(|p| enum_to_column(&p))
            .collect::<Result<_, _>>()?;

        sqlx::query(
            "INSERT INTO users (user_id, email, email_verified, phone_verified, kyc_verified, \
             terms_version, mfa_enrolled, trusted_device, risk_tier, compliance_checked_at, \
             auth_provider, roles, permissions) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        )
        .bind(new_user_id)
        .bind(new_profile.email())
        .bind(new_profile.email_verified())
        .bind(new_profile.phone_verified())
        .bind(new_profile.kyc_verified())
        .bind(new_profile.terms_version())
        .bind(new_profile.mfa_enrolled())
        .bind(new_profile.trusted_device())
        .bind(enum_to_column(&new_profile.risk_tier())?)
        .bind(new_profile.compliance_checked_at().map(to_i64))
        .bind(enum_to_column(&new_profile.auth_provider())?)
        .bind(&roles)
        .bind(&permissions)
        .execute(&mut *tx)
        .await
        .map_err(db_error("failed to insert user"))?;

        sqlx::query("INSERT INTO identity_links (issuer, subject, user_id) VALUES ($1, $2, $3)")
            .bind(issuer)
            .bind(subject)
            .bind(new_user_id)
            .execute(&mut *tx)
            .await
            .map_err(db_error("failed to insert identity link"))?;

        tx.commit()
            .await
            .map_err(db_error("failed to commit transaction"))?;

        Ok((
            new_user_id.to_string(),
            UserRecord {
                profile: new_profile,
                principal: new_principal,
            },
        ))
    }

    async fn get_user(&self, user_id: &str) -> Result<Option<UserRecord>, AuthError> {
        self.load_user(user_id).await
    }

    async fn save_user(&self, record: UserRecord) -> Result<(), AuthError> {
        let roles: Vec<String> = record
            .principal
            .roles_iter()
            .map(|r| enum_to_column(&r))
            .collect::<Result<_, _>>()?;
        let permissions: Vec<String> = record
            .principal
            .permissions_iter()
            .map(|p| enum_to_column(&p))
            .collect::<Result<_, _>>()?;

        sqlx::query(
            "INSERT INTO users (user_id, email, email_verified, phone_verified, kyc_verified, \
             terms_version, mfa_enrolled, trusted_device, risk_tier, compliance_checked_at, \
             auth_provider, roles, permissions) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             ON CONFLICT (user_id) DO UPDATE SET \
             email = EXCLUDED.email, email_verified = EXCLUDED.email_verified, \
             phone_verified = EXCLUDED.phone_verified, kyc_verified = EXCLUDED.kyc_verified, \
             terms_version = EXCLUDED.terms_version, mfa_enrolled = EXCLUDED.mfa_enrolled, \
             trusted_device = EXCLUDED.trusted_device, risk_tier = EXCLUDED.risk_tier, \
             compliance_checked_at = EXCLUDED.compliance_checked_at, \
             auth_provider = EXCLUDED.auth_provider, roles = EXCLUDED.roles, \
             permissions = EXCLUDED.permissions, updated_at = now()",
        )
        .bind(record.profile.user_id())
        .bind(record.profile.email())
        .bind(record.profile.email_verified())
        .bind(record.profile.phone_verified())
        .bind(record.profile.kyc_verified())
        .bind(record.profile.terms_version())
        .bind(record.profile.mfa_enrolled())
        .bind(record.profile.trusted_device())
        .bind(enum_to_column(&record.profile.risk_tier())?)
        .bind(record.profile.compliance_checked_at().map(to_i64))
        .bind(enum_to_column(&record.profile.auth_provider())?)
        .bind(&roles)
        .bind(&permissions)
        .execute(&self.pool)
        .await
        .map_err(db_error("failed to save user"))?;
        Ok(())
    }

    async fn insert_wallet(&self, wallet: WalletConnection) -> Result<(), AuthError> {
        sqlx::query(
            "INSERT INTO wallets (wallet_id, owner_user_id, chain, address, provider, verified_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(wallet.wallet_id())
        .bind(wallet.owner_user_id())
        .bind(enum_to_column(&wallet.chain())?)
        .bind(wallet.address())
        .bind(enum_to_column(&wallet.provider())?)
        .bind(to_i64(wallet.verified_at()))
        .execute(&self.pool)
        .await
        .map_err(|e| {
            // A UNIQUE(chain, address) violation means this exact on-chain
            // address is already linked to some wallet_id -- surfaced as a
            // policy violation rather than a raw SQL error.
            AuthError::PolicyViolation(format!("failed to insert wallet: {e}"))
        })?;
        Ok(())
    }

    async fn get_wallet(&self, wallet_id: &str) -> Result<Option<WalletConnection>, AuthError> {
        let row: Option<WalletRow> = sqlx::query_as(
            "SELECT wallet_id, owner_user_id, chain, address, provider, verified_at \
             FROM wallets WHERE wallet_id = $1",
        )
        .bind(wallet_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error("failed to load wallet"))?;
        row.map(WalletRow::into_wallet_connection).transpose()
    }

    async fn list_wallets(&self, owner_user_id: &str) -> Result<Vec<WalletConnection>, AuthError> {
        let rows: Vec<WalletRow> = sqlx::query_as(
            "SELECT wallet_id, owner_user_id, chain, address, provider, verified_at \
             FROM wallets WHERE owner_user_id = $1 ORDER BY verified_at DESC",
        )
        .bind(owner_user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_error("failed to list wallets"))?;
        rows.into_iter()
            .map(WalletRow::into_wallet_connection)
            .collect()
    }

    async fn create_kyc_case(&self, case: KycCaseRecord) -> Result<(), AuthError> {
        sqlx::query(
            "INSERT INTO kyc_cases (case_id, user_id, status, risk_tier, submitted_at, finalized_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&case.case_id)
        .bind(&case.user_id)
        .bind(case.status.as_str())
        .bind(case.risk_tier.map(|t| enum_to_column(&t)).transpose()?)
        .bind(to_i64(case.submitted_at))
        .bind(case.finalized_at.map(to_i64))
        .execute(&self.pool)
        .await
        .map_err(db_error("failed to insert kyc case"))?;
        Ok(())
    }

    async fn finalize_kyc_case(
        &self,
        case_id: &str,
        status: KycCaseStatus,
        risk_tier: Option<RiskTier>,
        finalized_at: u64,
    ) -> Result<(), AuthError> {
        let risk_tier = risk_tier.map(|t| enum_to_column(&t)).transpose()?;
        sqlx::query(
            "UPDATE kyc_cases SET status = $2, risk_tier = $3, finalized_at = $4 WHERE case_id = $1",
        )
        .bind(case_id)
        .bind(status.as_str())
        .bind(risk_tier)
        .bind(to_i64(finalized_at))
        .execute(&self.pool)
        .await
        .map_err(db_error("failed to finalize kyc case"))?;
        Ok(())
    }

    async fn get_kyc_case(&self, case_id: &str) -> Result<Option<KycCaseRecord>, AuthError> {
        let row: Option<KycCaseRow> = sqlx::query_as(
            "SELECT case_id, user_id, status, risk_tier, submitted_at, finalized_at \
             FROM kyc_cases WHERE case_id = $1",
        )
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error("failed to load kyc case"))?;
        row.map(KycCaseRow::into_record).transpose()
    }
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

type SetupOperationRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    serde_json::Value,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    i64,
);

fn setup_operation_row(row: SetupOperationRow) -> Result<AssetSetupOperation, AuthError> {
    let (
        operation_id,
        asset_id,
        issuer_user_id,
        network,
        mint_address,
        status,
        stages,
        mint_config_address,
        allow_list_address,
        block_list_address,
        thaw_extra_metas_address,
        error,
        created_at,
        updated_at,
    ) = row;
    Ok(AssetSetupOperation {
        operation_id,
        asset_id,
        issuer_user_id,
        network,
        mint_address,
        status,
        stages: serde_json::from_value(stages)
            .map_err(|_| AuthError::PolicyViolation("invalid stored setup stages".into()))?,
        mint_config_address,
        allow_list_address,
        block_list_address,
        thaw_extra_metas_address,
        error,
        created_at: to_u64(created_at),
        updated_at: to_u64(updated_at),
    })
}
