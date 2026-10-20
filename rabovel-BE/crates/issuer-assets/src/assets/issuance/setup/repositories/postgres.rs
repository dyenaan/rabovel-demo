use super::super::models::*;
use super::super::ports::SetupQueries;
use super::super::ports::SetupRepository;
use async_trait::async_trait;
use identity::AuthError;
use sqlx::PgPool;

pub struct PgSetups {
    pool: PgPool,
}
impl PgSetups {
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
#[async_trait]
impl SetupRepository for PgSetups {
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
}
#[async_trait::async_trait]
impl super::super::ports::SetupQueries for PgSetups {
    async fn get_asset_setup_operation(
        &self,
        asset_id: &str,
    ) -> Result<Option<AssetSetupOperation>, AuthError> {
        let row: Option<(String,String,String,String,String,String,serde_json::Value,Option<String>,Option<String>,Option<String>,Option<String>,Option<String>,i64,i64)> = sqlx::query_as("SELECT operation_id, asset_id, issuer_user_id, network, mint_address, status, stages, mint_config_address, allow_list_address, block_list_address, thaw_extra_metas_address, error, created_at, updated_at FROM asset_setup_operations WHERE asset_id=$1")
            .bind(asset_id).fetch_optional(&self.pool).await.map_err(db_error("failed to load asset setup operation"))?;
        row.map(setup_operation_row).transpose()
    }
}
