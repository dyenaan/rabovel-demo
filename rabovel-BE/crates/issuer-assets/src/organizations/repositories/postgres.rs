use super::super::models::*;
use super::super::ports::OrganizationRepository;
use async_trait::async_trait;
use identity::AuthError;
use sqlx::PgPool;

pub struct PgOrganizations {
    pool: PgPool,
}
impl PgOrganizations {
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
#[async_trait]
impl OrganizationRepository for PgOrganizations {
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
}
