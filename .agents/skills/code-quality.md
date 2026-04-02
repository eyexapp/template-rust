---
name: code-quality
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
  - clean code
  - naming
  - clippy
  - ownership
  - lifetime
  - trait
---

# Code Quality — Rust

## Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| Module | snake_case | `user_service.rs` |
| Struct | PascalCase | `UserService` |
| Function | snake_case | `find_by_email()` |
| Trait | PascalCase | `Repository` |
| Constant | UPPER_SNAKE | `MAX_RETRIES` |
| Type alias | PascalCase | `Result<T> = std::result::Result<T, AppError>` |
| Enum variant | PascalCase | `AppError::NotFound` |

## Ownership Best Practices

```rust
// Borrow when reading
fn validate_email(email: &str) -> bool { ... }

// Take ownership when consuming
fn create_user(dto: CreateUserDto) -> Result<User> { ... }

// Clone only when explicitly needed
let state = app_state.clone();

// Use Arc for shared state across handlers
struct AppState { pool: PgPool }
```

## Handler Pattern

```rust
pub async fn create_user(
    State(state): State<AppState>,
    Json(dto): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = UserService::create(&state.pool, dto).await?;
    Ok((StatusCode::CREATED, Json(user)))
}
```

- Axum extractors destructure request parts.
- Return `Result<impl IntoResponse, AppError>`.
- `?` operator for clean error propagation.

## Serde (Serialization)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
}
```

## Linting — Clippy

```bash
cargo clippy -- -W clippy::all -W clippy::pedantic
```

- `cargo clippy` — catches common mistakes, performance issues.
- `cargo fmt` — auto-format (rustfmt).
- `#[deny(clippy::unwrap_used)]` — force proper error handling.

## Anti-Patterns

- **No `.unwrap()` in production code** — use `?` or `.expect("reason")`.
- **No `String` for fixed sets** — use enums.
- **No `clone()` to avoid borrow checker** — restructure ownership.
- **No `Box<dyn Error>`** — use concrete `AppError` enum.
