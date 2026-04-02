---
name: testing
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
  - test
  - cargo test
  - mock
  - integration test
---

# Testing — Rust (cargo test)

## Unit Tests (In-Module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("alice@test.com"));
        assert!(!validate_email("invalid"));
    }

    #[tokio::test]
    async fn test_create_user() {
        let pool = test_pool().await;
        let dto = CreateUserDto { name: "Alice".into(), email: "a@b.com".into() };
        let user = UserService::create(&pool, dto).await.unwrap();
        assert_eq!(user.name, "Alice");
    }
}
```

## Integration Tests

```rust
// tests/api/users.rs
use axum::http::StatusCode;
use axum_test::TestServer;

#[tokio::test]
async fn test_create_user_endpoint() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/users")
        .json(&json!({"name": "Alice", "email": "a@b.com", "password": "pass"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let user: User = response.json();
    assert_eq!(user.name, "Alice");
}

#[tokio::test]
async fn test_get_user_not_found() {
    let server = TestServer::new(create_test_app().await).unwrap();
    let response = server.get("/api/users/nonexistent-id").await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}
```

## Test Database

```rust
async fn test_pool() -> PgPool {
    let url = std::env::var("TEST_DATABASE_URL").unwrap();
    let pool = PgPool::connect(&url).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    pool
}
```

## Mocking with Traits

```rust
#[async_trait]
trait UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
}

struct MockUserRepo { users: Vec<User> }

#[async_trait]
impl UserRepository for MockUserRepo {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        Ok(self.users.iter().find(|u| u.id == id).cloned())
    }
}
```

## Rules

- Unit tests in same file (`#[cfg(test)]` module).
- Integration tests in `tests/` directory.
- Use `axum_test::TestServer` for HTTP testing.
- Trait-based mocking — no external mock library needed.
