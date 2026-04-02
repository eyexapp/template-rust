use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::DatabaseConfig;

/// Create a `PostgreSQL` connection pool.
///
/// # Errors
///
/// Returns an error if the database connection cannot be established.
pub async fn create_pool(config: &DatabaseConfig) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await?;

    tracing::info!("Database connection pool created");
    Ok(pool)
}

/// Run pending `SQLx` migrations from the `migrations/` directory.
///
/// # Errors
///
/// Returns an error if any migration fails to apply.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("Database migrations applied");
    Ok(())
}

/// Quick health check — acquires and releases a connection.
pub async fn check_health(pool: &PgPool) -> bool {
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        sqlx::query("SELECT 1").fetch_one(pool),
    )
    .await
    .is_ok_and(|r| r.is_ok())
}
