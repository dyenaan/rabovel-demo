//! Runs against temporary tables only; never migrates the configured database.
use sqlx::{Connection, PgConnection};

#[tokio::test]
#[ignore = "requires RABOVEL_TEST_DATABASE_URL pointing to a local Postgres instance"]
async fn password_role_migration_removes_legacy_privileges_and_preserves_oidc() {
    let url = std::env::var("RABOVEL_TEST_DATABASE_URL").expect("set RABOVEL_TEST_DATABASE_URL");
    let mut connection = PgConnection::connect(&url)
        .await
        .expect("connect to test Postgres");
    let mut tx = connection.begin().await.unwrap();
    sqlx::raw_sql("CREATE TEMP TABLE users (user_id TEXT PRIMARY KEY, roles TEXT[], permissions TEXT[], updated_at TIMESTAMPTZ); CREATE TEMP TABLE password_credentials (user_id TEXT);
        INSERT INTO users VALUES
        ('password_admin', ARRAY['admin'], ARRAY['sign:withdrawal'], NULL),
        ('password_investor', ARRAY['trader'], ARRAY['view:portfolio'], NULL),
        ('password_issuer', ARRAY['issuer'], ARRAY['view:portfolio'], NULL),
        ('password_mixed', ARRAY['issuer','admin'], ARRAY['sign:withdrawal'], NULL),
        ('oidc_admin', ARRAY['admin'], ARRAY['sign:withdrawal'], NULL);
        INSERT INTO password_credentials VALUES ('password_admin'), ('password_investor'), ('password_issuer'), ('password_mixed');")
        .execute(&mut *tx).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../../../migrations/0006_restrict_password_roles.sql"
    ))
    .execute(&mut *tx)
    .await
    .unwrap();
    for (id, roles, permissions) in [
        (
            "password_admin",
            vec!["trader"],
            vec!["view:portfolio", "trade:spot"],
        ),
        (
            "password_investor",
            vec!["trader"],
            vec!["view:portfolio", "trade:spot"],
        ),
        ("password_issuer", vec!["issuer"], vec!["view:portfolio"]),
        ("password_mixed", vec!["issuer"], vec!["view:portfolio"]),
        ("oidc_admin", vec!["admin"], vec!["sign:withdrawal"]),
    ] {
        let actual: (Vec<String>, Vec<String>) =
            sqlx::query_as("SELECT roles, permissions FROM users WHERE user_id = $1")
                .bind(id)
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        assert_eq!(actual.0, roles, "roles for {id}");
        assert_eq!(actual.1, permissions, "permissions for {id}");
    }
    tx.rollback().await.unwrap();
}
