use super::super::models::*;
use super::super::ports::AccountRepository;
use crate::AuthError;
use crate::AuthPrincipal;
use crate::AuthProvider;
use crate::IdentityProfile;
use crate::RiskTier;
use crate::UserRole;
use crate::WalletAction;
use async_trait::async_trait;
use sqlx::PgPool;
use std::collections::HashSet;

pub struct PgAccounts {
    pool: PgPool,
}
impl PgAccounts {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
fn db_error(_context: &str) -> impl Fn(sqlx::Error) -> AuthError + '_ {
    |_| AuthError::ServiceUnavailable
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
impl PgAccounts {
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
impl AccountRepository for PgAccounts {
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
}
#[async_trait::async_trait]
impl super::super::ports::AccountQueries for PgAccounts {
    async fn get_user(&self, user_id: &str) -> Result<Option<UserRecord>, AuthError> {
        self.load_user(user_id).await
    }
}

#[async_trait::async_trait]
impl super::super::ports::ProfileUpdates for PgAccounts {
    async fn save_profile(&self, profile: IdentityProfile) -> Result<(), AuthError> {
        let updated = sqlx::query(
            "UPDATE users SET email=$2, email_verified=$3, phone_verified=$4, \
             kyc_verified=$5, terms_version=$6, mfa_enrolled=$7, trusted_device=$8, \
             risk_tier=$9, compliance_checked_at=$10, auth_provider=$11, updated_at=now() \
             WHERE user_id=$1",
        )
        .bind(profile.user_id())
        .bind(profile.email())
        .bind(profile.email_verified())
        .bind(profile.phone_verified())
        .bind(profile.kyc_verified())
        .bind(profile.terms_version())
        .bind(profile.mfa_enrolled())
        .bind(profile.trusted_device())
        .bind(enum_to_column(&profile.risk_tier())?)
        .bind(profile.compliance_checked_at().map(to_i64))
        .bind(enum_to_column(&profile.auth_provider())?)
        .execute(&self.pool)
        .await
        .map_err(db_error("failed to save profile"))?;
        if updated.rows_affected() == 0 {
            return Err(AuthError::Unauthorized);
        }
        Ok(())
    }
}
