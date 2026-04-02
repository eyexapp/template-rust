.PHONY: dev build release test lint fmt fmt-check check setup \
       db-migrate docker-up docker-down docker-build

# ── Development ──────────────────────────────────────────────────────────────

dev: ## Run in development mode
	cargo run

build: ## Build debug binary
	cargo build

release: ## Build optimized release binary
	cargo build --release

check: ## Fast compilation check
	cargo check

# ── Quality ──────────────────────────────────────────────────────────────────

test: ## Run all tests
	cargo test

lint: ## Run clippy with strict warnings
	cargo clippy -- -D warnings

fmt: ## Format code
	cargo fmt

fmt-check: ## Check formatting without modifying files
	cargo fmt -- --check

# ── Database ─────────────────────────────────────────────────────────────────

db-migrate: ## Run pending database migrations
	cargo sqlx migrate run

db-create-migration: ## Create a new migration (usage: make db-create-migration name=create_users)
	cargo sqlx migrate add $(name)

# ── Docker ───────────────────────────────────────────────────────────────────

docker-up: ## Start all containers
	docker compose up -d

docker-down: ## Stop all containers
	docker compose down

docker-build: ## Rebuild Docker image
	docker compose build

# ── Setup ────────────────────────────────────────────────────────────────────

setup: ## Initial project setup
	@echo "==> Copying .env.example to .env"
	@cp -n .env.example .env 2>/dev/null || true
	@echo "==> Building project"
	cargo build
	@echo "==> Setup complete! Run 'make dev' to start."

# ── Help ─────────────────────────────────────────────────────────────────────

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
