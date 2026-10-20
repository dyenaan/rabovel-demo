//! Shared connection and schema lifecycle; business SQL lives with features.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn connect(database_url: &str) -> Result<PgPool, String> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| format!("failed to connect to Postgres: {e}"))
}
pub async fn run_migrations(pool: &PgPool) -> Result<(), String> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| format!("failed to run database migrations: {e}"))
}
