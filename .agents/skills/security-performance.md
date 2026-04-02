---
name: security-performance
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
  - security
  - performance
  - async
  - memory safety
  - sql injection
  - tracing
---

# Security & Performance — Rust

## Performance

### Zero-Cost Abstractions

- Rust compile-time guarantees eliminate runtime overhead.
- No garbage collector — deterministic memory management.
- Axum handlers are async — efficient I/O with tokio.

### Connection Pooling

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url)
    .await?;
```

### Tracing (Observability)

```rust
use tracing::{info, warn, instrument};

#[instrument(skip(pool))]
pub async fn create_user(pool: &PgPool, dto: CreateUserDto) -> Result<User> {
    info!("Creating user: {}", dto.email);
    // ...
}
```

- `tracing` for structured logging — `info!`, `warn!`, `error!`.
- `#[instrument]` auto-creates spans for functions.
- `tower_http::trace::TraceLayer` for HTTP request tracing.

### Compile-Time Query Checking

- `sqlx::query_as!` validates SQL at compile time.
- Catches column type mismatches, missing columns, syntax errors.
- Zero runtime SQL parsing overhead.

## Security

### Memory Safety (Rust Guarantees)

- No buffer overflows — bounds checking at compile time.
- No null pointer dereference — `Option<T>` replaces null.
- No data races — ownership system prevents concurrent mutation.
- No use-after-free — borrow checker enforces lifetime rules.

### SQL Injection Prevention

- SQLx parameterizes all queries: `$1`, `$2` placeholders.
- Compile-time verified — impossible to forget parameterization.

### Authentication

```rust
// JWT validation extractor
let claims = decode::<Claims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::new(Algorithm::HS256),
)?;
```

### Secret Management

- `dotenvy` for `.env` loading (development only).
- Environment variables in production.
- `secrecy::SecretString` — zeroized on drop, excluded from Debug.

### CORS

```rust
let cors = CorsLayer::new()
    .allow_origin("https://myapp.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
```

### Dependencies

- `cargo audit` — check for known vulnerabilities.
- `cargo deny` — policy enforcement for licenses and advisories.
