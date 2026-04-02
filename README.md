# Rust Axum API Template

Production-ready, modular Axum API template with clean architecture, SQLx database layer, JWT authentication, structured error handling, and comprehensive testing.

## Tech Stack

| Category | Tool |
|----------|------|
| Framework | Axum 0.8 |
| Runtime | Tokio (async) |
| Database | PostgreSQL + SQLx (compile-time checked queries) |
| Error Handling | thiserror + anyhow |
| Auth | JWT (HS256) via jsonwebtoken |
| Logging | tracing + tracing-subscriber |
| Config | config crate + dotenvy |
| Testing | cargo test + reqwest (integration) |
| Linting | Clippy (pedantic + nursery) |
| Formatting | rustfmt |
| Container | Docker multi-stage (cargo-chef) + Compose |

## Quick Start

### Local Development

```bash
# Prerequisites: Rust (stable), PostgreSQL

# One-command setup
make setup

# Start the server
make dev

# Or directly:
cargo run
```

### Docker

```bash
docker compose up
# App: http://localhost:8080
# DB: localhost:5432
```

## Project Structure

```
├── src/
│   ├── main.rs                 # Entry point: tracing init, server start, graceful shutdown
│   ├── lib.rs                  # App builder, module tree
│   ├── config.rs               # Settings struct (env vars → typed config)
│   ├── error.rs                # AppError enum → JSON error responses
│   ├── state.rs                # AppState: DB pool + config (shared via Arc)
│   ├── db.rs                   # Pool creation, migrations, health check
│   ├── routes/
│   │   ├── mod.rs              # Router composition + middleware stack
│   │   └── health.rs           # GET /health, GET /health/ready
│   ├── middleware/
│   │   ├── mod.rs              # Middleware re-exports
│   │   └── auth.rs             # JWT Tower layer (disabled by default)
│   ├── extractors/
│   │   ├── mod.rs              # Extractor re-exports
│   │   ├── claims.rs           # JWT Claims from request extensions
│   │   └── json.rs             # AppJson<T> with better error messages
│   └── domain/
│       └── mod.rs              # ApiResponse<T>, Pagination, repository pattern
├── tests/
│   ├── common/
│   │   └── mod.rs              # TestApp, spawn_app() helper
│   └── health_test.rs          # Integration tests
├── migrations/                 # SQLx migrations (add .sql files here)
├── Cargo.toml                  # Dependencies + profiles + lint config
├── Makefile                    # Dev commands (make help for full list)
├── Dockerfile                  # Multi-stage production build (cargo-chef)
├── docker-compose.yml          # App + PostgreSQL
├── rust-toolchain.toml         # Pins stable Rust channel
├── rustfmt.toml                # Format settings
├── clippy.toml                 # Lint thresholds
└── .env.example                # Environment variables template
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Liveness check (always returns 200) |
| GET | `/health/ready` | Readiness check (tests DB connection) |

## Adding New Features

### Create a Route Handler

```rust
// src/routes/users.rs
use axum::extract::State;
use axum::Json;

use crate::domain::ApiResponse;
use crate::error::Result;
use crate::state::AppState;

pub async fn list_users(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<User>>>> {
    let users = sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(&state.db)
        .await?;

    Ok(Json(ApiResponse::new(users)))
}
```

Register in `src/routes/mod.rs`:
```rust
let user_routes = Router::new()
    .route("/", axum::routing::get(users::list_users));

Router::new()
    .nest("/health", health_routes)
    .nest("/api/v1/users", user_routes)
```

### Create a Migration

```bash
# Install sqlx-cli (one-time)
cargo install sqlx-cli --no-default-features --features postgres

# Create migration
cargo sqlx migrate add create_users

# Edit the generated file in migrations/
# Run it
cargo sqlx migrate run
```

Example migration (`migrations/YYYYMMDDHHMMSS_create_users.sql`):
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Create an Extractor

```rust
// src/extractors/pagination.rs
use axum::extract::Query;

use crate::domain::PaginationParams;

// Already available — use it in handlers:
pub async fn list_items(Query(pagination): Query<PaginationParams>) -> impl IntoResponse {
    let offset = pagination.offset();
    let limit = pagination.per_page;
    // ...
}
```

### Create a Service / Repository

```rust
// src/services/user_service.rs
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;

pub struct UserService<'a> {
    db: &'a PgPool,
}

impl<'a> UserService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(self.db)
            .await?;
        Ok(user)
    }
}
```

## Testing

```bash
# Run all tests
make test
# or: cargo test

# Run specific test
cargo test test_health_returns_200

# Run with output
cargo test -- --nocapture

# Run only integration tests
cargo test --test health_test
```

## Linting & Formatting

```bash
# Check lint issues (pedantic + nursery)
make lint
# or: cargo clippy -- -D warnings

# Format code
make fmt
# or: cargo fmt

# Check formatting
make fmt-check
# or: cargo fmt -- --check
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server port |
| `DATABASE_URL` | — | PostgreSQL connection URL (required) |
| `JWT_SECRET` | — | JWT signing secret (required) |
| `JWT_EXPIRATION_SECS` | `3600` | JWT token TTL in seconds |
| `RUST_LOG` | `app=info,tower_http=info` | Log filter directive |

You can also use the `APP_` prefix with `__` separator for nested config:
- `APP_SERVER__HOST` → `settings.server.host`
- `APP_DATABASE__MAX_CONNECTIONS` → `settings.database.max_connections`

## Enabling JWT Authentication

Uncomment the JWT layer in `src/routes/mod.rs`:

```rust
use crate::middleware::auth::JwtAuthLayer;

// In create_router():
app.layer(JwtAuthLayer::new(
    &state.config.auth,
    vec!["/health".into(), "/health/ready".into()],
))
```

Protected routes can extract claims:

```rust
use crate::extractors::claims::AuthClaims;

async fn protected_handler(AuthClaims(claims): AuthClaims) -> impl IntoResponse {
    format!("Hello, user {}", claims.sub)
}
```

## Make Commands

```bash
make help         # Show all available commands
make setup        # Initial project setup
make dev          # Run in development mode
make build        # Build debug binary
make release      # Build optimized release binary
make test         # Run all tests
make lint         # Run clippy with strict warnings
make fmt          # Format code
make docker-up    # Start Docker containers
make docker-down  # Stop Docker containers
```

## License

MIT
