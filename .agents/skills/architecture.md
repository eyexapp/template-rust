---
name: architecture
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
  - architecture
  - axum
  - sqlx
  - tower
  - middleware
  - ownership
  - borrow
---

# Architecture — Rust (Axum + SQLx + Tower)

## Axum Server

```rust
use axum::{Router, routing::get};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();

    let app = Router::new()
        .nest("/api/users", user_routes())
        .nest("/api/auth", auth_routes())
        .layer(CorsLayer::permissive())
        .with_state(AppState { pool });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Project Structure

```
src/
├── main.rs             ← Server setup, router composition
├── config.rs           ← Environment config (envy/dotenvy)
├── error.rs            ← AppError type, IntoResponse impl
├── state.rs            ← AppState (shared state for handlers)
├── handlers/           ← Axum handlers (extract → call service → respond)
│   ├── mod.rs
│   ├── user.rs
│   └── auth.rs
├── services/           ← Business logic
│   ├── mod.rs
│   └── user.rs
├── repositories/       ← SQLx queries
│   ├── mod.rs
│   └── user.rs
├── models/             ← Domain types + DB models
│   ├── mod.rs
│   └── user.rs
├── middleware/          ← Tower middleware (auth, logging)
│   └── auth.rs
└── extractors/         ← Custom Axum extractors
    └── auth.rs
```

## SQLx (Compile-Time Checked Queries)

```rust
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<User, AppError> {
    sqlx::query_as!(User, "SELECT id, name, email, created_at FROM users WHERE id = $1", id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("User"))
}
```

- Queries validated at compile time against the database schema.
- `sqlx::query_as!` maps rows directly to structs.
- Zero-cost abstraction — no ORM overhead.

## Error Handling — thiserror + anyhow

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Validation: {0}")]
    Validation(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(r) => (StatusCode::NOT_FOUND, format!("{r} not found")),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
            AppError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}
```

## Tower Middleware

```rust
// Auth middleware as extractor
pub struct AuthUser(pub Claims);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts.headers.get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;
        let claims = decode_jwt(token)?;
        Ok(AuthUser(claims))
    }
}
```

## Rules

- Handlers are thin — extract data, call service, return response.
- SQLx compile-time checked queries — no raw string SQL.
- `AppError` implements `IntoResponse` — clean error propagation with `?`.
- Ownership/borrowing: pass `&PgPool`, clone `Arc<AppState>`.
