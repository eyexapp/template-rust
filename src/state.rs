use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Settings;

/// Shared application state, passed to all handlers via Axum's `State` extractor.
///
/// This struct is cheaply cloneable — `PgPool` is already `Arc`-based internally,
/// and `Settings` is wrapped in `Arc`.
#[derive(Debug, Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Settings>,
}
