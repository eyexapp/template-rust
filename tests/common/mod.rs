use tokio::net::TcpListener;

/// Test application wrapper providing a running server instance and HTTP client.
pub struct TestApp {
    pub address: String,
    #[allow(dead_code)]
    pub port: u16,
    pub client: reqwest::Client,
}

/// Spawn the application on a random port and return a `TestApp` connected to it.
///
/// This starts a real HTTP server — use it for integration tests. The server
/// runs without a database connection, so health/ready will return 503.
///
/// For tests that require a database, set `DATABASE_URL` in the test env
/// and use `app::build_app()` directly.
pub async fn spawn_app() -> TestApp {
    // Use port 0 to let the OS assign a random available port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    // Build a minimal router (no DB — health/ready will fail gracefully)
    let state = minimal_state();
    let router = app::routes::create_router(state);

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("Server failed");
    });

    TestApp {
        address,
        port,
        client: reqwest::Client::new(),
    }
}

/// Create a minimal `AppState` for tests without a database.
///
/// Uses a null-like pool that will fail on actual queries — suitable for
/// testing routes that don't touch the database (e.g., `/health`).
fn minimal_state() -> app::state::AppState {
    use std::sync::Arc;

    // Set minimal env for config
    std::env::set_var("DATABASE_URL", "postgres://localhost/test_not_connected");
    std::env::set_var("JWT_SECRET", "test-secret-key");

    let settings = app::config::Settings::load();

    // Create a pool that won't actually connect — tests that need DB
    // should override this with a real pool.
    let pool_options = sqlx::postgres::PgPoolOptions::new().max_connections(1);
    let pool = pool_options
        .connect_lazy("postgres://localhost/test_not_connected")
        .expect("Failed to create lazy pool");

    app::state::AppState {
        db: pool,
        config: Arc::new(settings),
    }
}
