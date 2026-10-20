use super::super::models::*;
use super::super::ports::CaseRepository;
use async_trait::async_trait;
use identity::AuthError;
use identity::RiskTier;
use sqlx::PgPool;

pub struct PgCases {
    pool: PgPool,
}
impl PgCases {
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
#[async_trait]
impl CaseRepository for PgCases {
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
