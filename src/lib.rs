pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod extractors;
pub mod middleware;
pub mod routes;
pub mod state;

use std::sync::Arc;

use axum::Router;

use crate::config::Settings;
use crate::state::AppState;

/// Build the complete application.
///
/// Creates a database pool, runs migrations, and assembles the Axum router
/// with all middleware attached.
///
/// # Errors
///
/// Returns an error if the database pool cannot be created or migrations fail.
pub async fn build_app(settings: Settings) -> anyhow::Result<Router> {
    // Database
    let pool = db::create_pool(&settings.database).await?;
    db::run_migrations(&pool).await?;

    // State
    let state = AppState {
        db: pool,
        config: Arc::new(settings),
    };

    // Router
    let router = routes::create_router(state);

    Ok(router)
}
