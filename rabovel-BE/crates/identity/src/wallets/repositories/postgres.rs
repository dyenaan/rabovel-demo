use super::super::ports::WalletRepository;
use crate::AuthError;
use crate::SupportedChain;
use crate::WalletConnection;
use crate::WalletProvider;
use async_trait::async_trait;
use sqlx::PgPool;

pub struct PgWallets {
    pool: PgPool,
}
impl PgWallets {
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
#[async_trait]
impl WalletRepository for PgWallets {
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
}
#[async_trait::async_trait]
impl super::super::ports::WalletQueries for PgWallets {
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
}
