//! Optional integration check using connection-local temporary tables only.
use identity::accounts::ports::ProfileUpdates;
use identity::accounts::repositories::postgres::PgAccounts;
use identity::BrokerageAuthService;
use identity::BrokerageSecurityPolicy;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires RABOVEL_TEST_DATABASE_URL for isolated temporary-table verification"]
async fn profile_updates_preserve_postgres_roles_and_permissions() {
    let url = std::env::var("RABOVEL_TEST_DATABASE_URL").expect("set RABOVEL_TEST_DATABASE_URL");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .expect("connect test Postgres");
    sqlx::raw_sql("CREATE TEMP TABLE users(user_id TEXT PRIMARY KEY,email TEXT,email_verified BOOLEAN,phone_verified BOOLEAN,kyc_verified BOOLEAN,terms_version TEXT,mfa_enrolled BOOLEAN,trusted_device BOOLEAN,risk_tier TEXT,compliance_checked_at BIGINT,auth_provider TEXT,roles TEXT[],permissions TEXT[],updated_at TIMESTAMPTZ); INSERT INTO users(user_id,roles,permissions) VALUES ('usr_profile',ARRAY['issuer'],ARRAY['view:portfolio']);").execute(&pool).await.unwrap();
    let mut profile = BrokerageAuthService::new(BrokerageSecurityPolicy::default())
        .profile_from_email_password("usr_profile", "issuer@example.com")
        .unwrap();
    profile.accept_terms("new-terms").unwrap();
    PgAccounts::new(pool.clone())
        .save_profile(profile)
        .await
        .unwrap();
    let row: (Vec<String>, Vec<String>, String) = sqlx::query_as(
        "SELECT roles,permissions,terms_version FROM users WHERE user_id='usr_profile'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        row,
        (
            vec!["issuer".to_string()],
            vec!["view:portfolio".to_string()],
            "new-terms".into()
        )
    );
    pool.close().await;
}
