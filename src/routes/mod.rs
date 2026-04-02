pub mod health;

use std::time::Duration;

use axum::http::StatusCode;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Build the application router with all routes and middleware layers.
pub fn create_router(state: AppState) -> Router {
    // --- Routes ---
    let health_routes = Router::new()
        .route("/", axum::routing::get(health::health))
        .route("/ready", axum::routing::get(health::ready));

    // --- Compose ---
    let app = Router::new()
        .nest("/health", health_routes)
        // Add new route groups here:
        // .nest("/api/v1/users", user_routes)
        .with_state(state);

    // --- Middleware (applied bottom-up: last added runs first) ---
    app.layer(TimeoutLayer::with_status_code(
        StatusCode::GATEWAY_TIMEOUT,
        Duration::from_secs(30),
    ))
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http())
    // Uncomment to enable JWT authentication:
    // .layer(JwtAuthLayer::new(
    //     &state.config.auth,
    //     vec!["/health".into(), "/health/ready".into()],
    // ))
}
