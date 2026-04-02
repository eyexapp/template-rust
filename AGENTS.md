# AGENTS.md — Rust Axum API

## Project Identity

| Key | Value |
|-----|-------|
| Language | Rust (stable) |
| Framework | Axum (Tower-based) |
| Runtime | Tokio (full features) |
| Database | PostgreSQL + SQLx (compile-time checked) |
| Error Handling | thiserror + anyhow |
| Logging | tracing + tracing-subscriber |
| Auth | JWT (HS256, disabled by default) |
| Config | config crate + dotenvy |
| Testing | cargo test + integration tests |
| Linting | Clippy (pedantic + nursery) |

---

## Architecture — Layered Axum API

```
src/
├── main.rs              ← Entry point: tracing, config, server startup, graceful shutdown
├── lib.rs               ← Module tree + build_app()
├── config.rs            ← Settings struct (server, database, auth) from env vars
├── error.rs             ← AppError enum → IntoResponse (JSON errors)
├── state.rs             ← AppState: PgPool + Arc<Settings>
├── db.rs                ← Pool creation, migration runner, health check
├── routes/
│   ├── mod.rs           ← Router composition + Tower middleware stack
│   ├── health.rs        ← Health/readiness endpoints
│   └── items.rs         ← Feature CRUD handlers
├── middleware/
│   ├── mod.rs           ← Middleware re-exports
│   └── auth.rs          ← JWT Bearer validation (Tower Layer + Service)
├── extractors/
│   ├── mod.rs           ← Extractor re-exports
│   ├── claims.rs        ← AuthClaims from request extensions
│   └── json.rs          ← AppJson<T> with validation errors
└── domain/
    └── mod.rs           ← ApiResponse<T>, PaginationParams, repository traits
```

### Request Flow
```
Request → Tokio → Tower Middleware Stack → Axum Router → Handler → Response
                    ├── TraceLayer (structured request logging)
                    ├── CorsLayer (permissive CORS)
                    ├── TimeoutLayer (30s)
                    └── JwtAuthLayer (optional)
```

### Strict Layer Rules

| Layer | Can Import From | NEVER Imports |
|-------|----------------|---------------|
| `routes/` | state, domain, error, extractors | middleware internals |
| `middleware/` | state, error, config | routes/ |
| `extractors/` | error, domain | routes/, middleware/ |
| `domain/` | (pure types — no framework deps) | routes/, middleware/, state |
| `state.rs` | config, db | routes/ |

---

## Adding New Code — Where Things Go

### New Endpoint
1. Create handler function in `src/routes/` (new file or add to existing)
2. Add `pub mod name;` in `src/routes/mod.rs`
3. Register route in `create_router()` with `.route()` or `.nest()`
4. Use `State<AppState>` for DB access
5. Return `Result<impl IntoResponse>` (errors auto-convert via `?`)
6. Write integration test in `tests/`

### New Database Table
1. `cargo sqlx migrate add create_table_name`
2. Edit `.sql` file in `migrations/`
3. `cargo sqlx migrate run`
4. Create Rust struct with `#[derive(sqlx::FromRow)]`

### Handler Pattern
```rust
use axum::extract::{State, Path, Json};
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
        .ok_or_else(|| AppError::NotFound(format!("Item {id} not found")))?;

    Ok(Json(ApiResponse::new(item)))
}
```

### Custom Extractor
```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct CurrentUser(pub User);

impl<S: Send + Sync> FromRequestParts<S> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract from extensions
    }
}
```

---

## Design & Architecture Principles

### Ownership & Borrowing — Idiomatic Patterns
- `AppState` is `Clone` because `PgPool` is `Arc`-based internally
- Use `Arc<Settings>` for shared config — clone is cheap
- Handlers receive extractors by value — Axum moves them in
- `?` operator for error propagation — clean, readable chains

### Error Type Design
```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation: {0}")]
    Validation(String),

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()),
        };
        // Log internal details, return sanitized JSON to client
        (status, Json(ErrorResponse { error: message })).into_response()
    }
}
```

### Tower Middleware Composition
```rust
// src/routes/mod.rs
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/items", item_routes())
        .route("/health", get(health))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
}
```

---

## Error Handling

### The `?` Operator Chain
- All handlers return `Result<impl IntoResponse>`
- `sqlx::Error` → `AppError::Database` via `#[from]` — automatic
- `anyhow::Error` → `AppError::Internal` via `#[from]` — catch-all
- Custom domain errors: use named variants (`NotFound`, `Validation`, `Conflict`)
- Internal details are LOGGED but NEVER exposed to clients

### Graceful Shutdown
```rust
// main.rs — built-in
tokio::signal::ctrl_c().await // waits for signal, then graceful shutdown
```

---

## Code Quality

### Naming Conventions
| Artifact | Convention | Example |
|----------|-----------|---------|
| Module | `snake_case.rs` | `item_handler.rs` |
| Struct | `PascalCase` | `AppState`, `ProductItem` |
| Function | `snake_case` | `get_item`, `create_router` |
| Trait | `PascalCase` verb | `Repository`, `IntoResponse` |
| Constant | `SCREAMING_SNAKE` | `MAX_CONNECTIONS` |
| Migration | `YYYYMMDDHHMMSS_desc.sql` | `20240101120000_create_items.sql` |

### Clippy — Pedantic + Nursery
```toml
# Cargo.toml
[lints.clippy]
pedantic = "warn"
nursery = "warn"
```
- All Clippy warnings must be resolved
- `unsafe` code is **forbidden** via `[lints.rust]`

---

## Testing Strategy

| Level | What | Where | Approach |
|-------|------|-------|----------|
| Unit | Pure functions, domain logic | `#[cfg(test)] mod tests` in source | In-module tests |
| Integration | HTTP endpoints | `tests/` directory | `TestApp` + `spawn_app()` |

### Integration Test Pattern
```rust
// tests/common/mod.rs
pub struct TestApp {
    pub addr: String,
    pub client: reqwest::Client,
}

pub async fn spawn_app() -> TestApp {
    let app = build_app().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(axum::serve(listener, app).into_future());
    TestApp { addr, client: reqwest::Client::new() }
}
```

### What MUST Be Tested
- All handler functions: success + error paths
- Custom extractors: valid + invalid input
- Error type → HTTP status code mapping
- Database operations: CRUD + edge cases
- Health check: with and without DB

---

## Security & Performance

### Security
- JWT auth disabled by default — enable by uncommenting Layer in `routes/mod.rs`
- `AppError::IntoResponse` NEVER exposes internal errors to clients
- SQLx compile-time checked queries — SQL injection impossible
- `unsafe` forbidden at the lint level

### Performance
- Tokio async runtime — non-blocking I/O
- `PgPool` with connection pooling (configurable max connections)
- Tower middleware stack — zero-cost abstractions (compile-time dispatch)
- Health check has 5s timeout — prevents hanging on DB issues
- Multi-stage Docker build — minimal production image

---

## Commands

| Action | Command |
|--------|---------|
| Dev | `cargo run` |
| Build release | `cargo build --release` |
| Test | `cargo test` |
| Lint | `cargo clippy` |
| Format | `cargo fmt` |
| Add migration | `cargo sqlx migrate add <name>` |
| Run migrations | `cargo sqlx migrate run` |
| Check queries | `cargo sqlx prepare` |
| Docker | `make docker-up` |

---

## Prohibitions — NEVER Do These

1. **NEVER** use `unsafe` code — forbidden at lint level
2. **NEVER** use `.unwrap()` in production code — use `?` operator or `expect()` with context
3. **NEVER** expose internal error details to clients — log internally, return sanitized JSON
4. **NEVER** use `String` concatenation for SQL — sqlx parameterized queries only
5. **NEVER** skip Clippy warnings — pedantic + nursery lints are enforced
6. **NEVER** use `tokio::spawn` without proper error handling on the spawned task
7. **NEVER** block the async runtime — no `std::thread::sleep`, use `tokio::time::sleep`
8. **NEVER** use global mutable state — pass `AppState` via Axum's `State` extractor
9. **NEVER** ignore `#[must_use]` returns — Results must be handled
10. **NEVER** use `panic!` for expected errors — use `AppError` variants + `?`
