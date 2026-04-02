# Rust Axum API Template — AI Agent Instructions

## Project Overview

This is a production-grade Axum API template using Rust (stable). It follows clean architecture principles with separated layers for routes, middleware, extractors, domain types, and database access. Tower middleware stack provides cross-cutting concerns.

## Architecture

### Request Flow

```
Request → Tokio Runtime → Tower Middleware Stack → Axum Router → Handler → Response
                           ├── TraceLayer (structured request logging)
                           ├── CorsLayer (permissive CORS)
                           ├── TimeoutLayer (30s gateway timeout)
                           └── JwtAuthLayer (optional, disabled by default)
```

### Layer Responsibilities

- **Routes** (`src/routes/`): HTTP handler functions. Receive extractors (State, Json, Query, Path), call services/DB, return `impl IntoResponse`. Keep thin — delegate business logic.
- **Middleware** (`src/middleware/`): Tower `Layer` + `Service` implementations. Cross-cutting concerns (auth, rate limiting). Applied in `src/routes/mod.rs`.
- **Extractors** (`src/extractors/`): Custom Axum extractors implementing `FromRequest` or `FromRequestParts`. Transform raw request data into typed structures.
- **Domain** (`src/domain/`): Shared types: `ApiResponse<T>`, `PaginationParams`, `PaginatedResponse<T>`, repository traits. Pure data structures, no framework dependency.
- **Error** (`src/error.rs`): `AppError` enum with `thiserror`. Implements `IntoResponse` → structured JSON. All handler errors flow through this.
- **State** (`src/state.rs`): `AppState` struct shared across all handlers via Axum's `State<AppState>` extractor. Contains `PgPool` + `Arc<Settings>`.
- **Config** (`src/config.rs`): `Settings` struct loaded from env vars via `config` crate + `dotenvy`. Hierarchical: defaults → env vars.
- **Database** (`src/db.rs`): Pool creation, migration runner, health check. Uses SQLx with PostgreSQL.

### Key Patterns

#### Handler Function

```rust
use axum::extract::{State, Path};
use axum::Json;
use uuid::Uuid;

use crate::domain::ApiResponse;
use crate::error::Result;
use crate::state::AppState;

pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Item>>> {
    let item = sqlx::query_as!(Item, "SELECT * FROM items WHERE id = $1", id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("Item {id} not found")))?;

    Ok(Json(ApiResponse::new(item)))
}
```

#### Custom Extractor

```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct CurrentUser(pub User);

impl<S: Send + Sync> FromRequestParts<S> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract from extensions, headers, etc.
    }
}
```

#### Error Propagation

All fallible operations use the `?` operator with `AppError`. Errors are automatically converted:
- `sqlx::Error` → `AppError::Database` → 500 JSON
- `anyhow::Error` → `AppError::Internal` → 500 JSON
- Custom errors → appropriate HTTP status + JSON body

Internal error details are logged but never exposed to clients.

## File Naming Conventions

- All Rust files: `snake_case.rs`
- Module directories have `mod.rs` for re-exports
- Test files: `tests/<name>_test.rs` (integration) or `#[cfg(test)] mod tests` (unit)
- Migrations: `YYYYMMDDHHMMSS_description.sql`

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point: tracing init, config load, server startup, graceful shutdown |
| `src/lib.rs` | Module tree + `build_app()` function |
| `src/config.rs` | `Settings` struct with nested config (server, database, auth) |
| `src/error.rs` | `AppError` enum → `IntoResponse` for JSON error responses |
| `src/state.rs` | `AppState` struct (PgPool + Arc<Settings>) |
| `src/db.rs` | Database pool, migrations, health check |
| `src/routes/mod.rs` | Router composition + Tower middleware stack |
| `src/routes/health.rs` | Health/readiness endpoints |
| `src/middleware/auth.rs` | JWT Bearer token validation (Tower Layer) |
| `src/extractors/claims.rs` | `AuthClaims` extractor from request extensions |
| `src/extractors/json.rs` | `AppJson<T>` with descriptive validation errors |
| `src/domain/mod.rs` | `ApiResponse<T>`, `PaginationParams`, repository trait pattern |
| `tests/common/mod.rs` | `TestApp` + `spawn_app()` for integration tests |

## Common Tasks

### Add a new endpoint
1. Create handler function in `src/routes/` (new file or existing)
2. If new file: add `pub mod name;` in `src/routes/mod.rs`
3. Register route in `create_router()` with `.route()` or `.nest()`
4. Use `State<AppState>` for DB access, `Result<impl IntoResponse>` as return type
5. Write integration test in `tests/`

### Add a database table
1. `cargo sqlx migrate add create_table_name`
2. Edit the `.sql` file in `migrations/`
3. `cargo sqlx migrate run`
4. Create Rust struct with `sqlx::FromRow` derive

### Add a custom extractor
1. Create file in `src/extractors/`
2. Implement `FromRequest<AppState>` or `FromRequestParts<AppState>`
3. Return `AppError` as rejection type for consistent error handling
4. Add `pub mod name;` in `src/extractors/mod.rs`

### Add middleware
1. Create file in `src/middleware/`
2. Implement Tower `Layer<S>` + `Service<Request<Body>>`
3. Add `.layer(YourLayer::new(...))` in `src/routes/mod.rs`

## Testing

- **Unit tests**: `#[cfg(test)] mod tests` blocks within source files
- **Integration tests**: `tests/` directory, uses `spawn_app()` to start real HTTP server
- **Test helpers**: `tests/common/mod.rs` — `TestApp`, `spawn_app()`, lazy DB pool
- **Run**: `cargo test` or `make test`
- **No DB needed**: Integration tests use a lazy pool, health/ready returns 503 without DB

## Dependencies

- **axum** — Web framework (Tower-based, modular routing)
- **tokio** — Async runtime (full features)
- **tower + tower-http** — Middleware: CORS, tracing, timeout, request-id
- **sqlx** — Async database toolkit (PostgreSQL, compile-time checked queries, migrations)
- **serde + serde_json** — Serialization / deserialization
- **thiserror** — Derive macro for custom error types
- **anyhow** — Flexible error propagation
- **tracing + tracing-subscriber** — Structured logging with span support
- **config + dotenvy** — Layered configuration from env vars
- **jsonwebtoken** — JWT encoding / decoding (HS256)
- **uuid** — UUID v4 generation + serde support
- **chrono** — Date/time with serde support

## Important Notes

- JWT auth is **disabled by default** — uncomment layer in `src/routes/mod.rs`
- `unsafe` code is **forbidden** via `[lints.rust]` in Cargo.toml
- Clippy runs with `pedantic` + `nursery` lints — high code quality bar
- SQLx migrations run automatically on startup in `build_app()`
- `AppState` is `Clone` because `PgPool` is `Arc`-based internally
- Graceful shutdown via `tokio::signal::ctrl_c` in `main.rs`
- Health check has 5s timeout to prevent hanging on DB connection issues
