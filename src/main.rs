use tokio::net::TcpListener;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use app::config::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "app=info,tower_http=info".into()),
        )
        .with(fmt::layer())
        .init();

    // Load configuration
    let settings = Settings::load();
    let addr = format!("{}:{}", settings.server.host, settings.server.port);

    // Build application
    let app = app::build_app(settings).await?;

    // Start server
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");
    tracing::info!("Shutdown signal received, starting graceful shutdown");
}
