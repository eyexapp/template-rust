use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::state::AppState;

/// `GET /health` — Basic liveness check (always returns 200).
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// `GET /health/ready` — Readiness check (verifies database connectivity).
///
/// Returns 200 if the database is reachable, 503 otherwise.
pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    if crate::db::check_health(&state.db).await {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ok",
                "database": "connected",
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "error",
                "database": "unavailable",
            })),
        )
    }
}
