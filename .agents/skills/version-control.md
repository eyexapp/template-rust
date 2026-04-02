---
name: version-control
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
  - git
  - commit
  - ci
  - cargo
  - deploy
---

# Version Control — Rust

## Commits (Conventional)

- `feat(users): add pagination with cursor`
- `fix(auth): validate JWT audience claim`
- `refactor(handlers): extract common extractor`

## CI Pipeline

1. `cargo fmt --check` — format check
2. `cargo clippy -- -D warnings` — lint (fail on warnings)
3. `cargo test` — unit + integration tests
4. `cargo build --release` — optimized build
5. `cargo sqlx prepare` — save offline query data for CI

## Cargo.toml

```toml
[package]
name = "myapp"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
serde = { version = "1", features = ["derive"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
tracing = "0.1"
```

## .gitignore

```
/target
.env
*.pdb
```

## Docker (Multi-Stage)

```dockerfile
FROM rust:1.83 AS build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build /app/target/release/myapp /myapp
EXPOSE 3000
CMD ["/myapp"]
```

## SQLx Offline Mode

```bash
cargo sqlx prepare           # Save query metadata
cargo sqlx prepare --check   # Verify in CI (no DB needed)
```
