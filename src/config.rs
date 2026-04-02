use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration_secs: u64,
}

fn default_host() -> String {
    "0.0.0.0".to_owned()
}

const fn default_port() -> u16 {
    8080
}

const fn default_max_connections() -> u32 {
    10
}

const fn default_jwt_expiration() -> u64 {
    3600
}

impl Settings {
    /// Load settings from environment variables.
    ///
    /// Reads `.env` file if present, then maps environment variables with
    /// the `APP_` prefix into nested config using `__` as separator.
    ///
    /// Example: `APP_SERVER__PORT=3000` → `settings.server.port = 3000`
    ///
    /// # Panics
    ///
    /// Panics if required configuration is missing or invalid.
    #[must_use]
    pub fn load() -> Self {
        // Load .env file (ignore if missing)
        let _ = dotenvy::dotenv();

        Config::builder()
            .set_default("server.host", default_host())
            .expect("default host")
            .set_default("server.port", i64::from(default_port()))
            .expect("default port")
            .set_default(
                "database.max_connections",
                i64::from(default_max_connections()),
            )
            .expect("default max_connections")
            .set_default(
                "auth.jwt_expiration_secs",
                i64::try_from(default_jwt_expiration()).expect("jwt expiration fits i64"),
            )
            .expect("default jwt_expiration_secs")
            // Map: APP_SERVER__HOST → server.host
            // Map: APP_DATABASE__URL → database.url
            // Map: APP_AUTH__JWT_SECRET → auth.jwt_secret
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            // Also support flat env vars (DATABASE_URL, JWT_SECRET, etc.)
            .set_override_option("database.url", std::env::var("DATABASE_URL").ok())
            .expect("database url override")
            .set_override_option("auth.jwt_secret", std::env::var("JWT_SECRET").ok())
            .expect("jwt secret override")
            .set_override_option(
                "auth.jwt_expiration_secs",
                std::env::var("JWT_EXPIRATION_SECS").ok(),
            )
            .expect("jwt expiration override")
            .set_override_option("server.host", std::env::var("HOST").ok())
            .expect("host override")
            .set_override_option("server.port", std::env::var("PORT").ok())
            .expect("port override")
            .build()
            .expect("Failed to build configuration")
            .try_deserialize()
            .expect("Failed to deserialize configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_are_applied() {
        // Clear any env vars that might interfere
        std::env::remove_var("APP_SERVER__HOST");
        std::env::remove_var("APP_SERVER__PORT");
        std::env::remove_var("HOST");
        std::env::remove_var("PORT");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("JWT_SECRET");
        std::env::remove_var("APP_DATABASE__URL");
        std::env::remove_var("APP_AUTH__JWT_SECRET");

        // Set required fields
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("JWT_SECRET", "test-secret");

        let settings = Settings::load();

        assert_eq!(settings.server.host, "0.0.0.0");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.database.max_connections, 10);
        assert_eq!(settings.auth.jwt_expiration_secs, 3600);

        // Cleanup
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("JWT_SECRET");
    }
}
