# RustVault — Command Runner
# https://just.systems

set dotenv-load

# Default: show available recipes
default:
    @just --list

# ── Development ──────────────────────────────────────────────

# Start backend and frontend in parallel
dev:
    #!/usr/bin/env bash
    trap 'kill 0' EXIT
    just dev-backend &
    just dev-frontend &
    wait

# Start Rust backend with auto-reload
dev-backend:
    cargo watch -x run -w crates/

# Start Vite dev server
dev-frontend:
    cd web && bun run dev

# ── Build ────────────────────────────────────────────────────

# Build all (backend release + frontend)
build: build-backend build-frontend

# Build Rust backend (release)
build-backend:
    cargo build --release

# Build frontend assets
build-frontend:
    cd web && bun run build

# ── Test ─────────────────────────────────────────────────────

# Run all tests
test: test-backend test-frontend

# Run Rust tests only
test-backend:
    cargo test --workspace

# Run frontend type checks
test-frontend:
    cd web && bun run typecheck

# ── Lint & Format ────────────────────────────────────────────

# Run clippy and frontend lint
lint:
    cargo clippy --workspace --all-targets -- -D warnings
    cd web && bun run lint 2>/dev/null || true

# Format all code
fmt:
    cargo fmt --all
    cd web && bunx prettier --write src/ 2>/dev/null || true

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# ── Database ─────────────────────────────────────────────────

# Run database migrations
migrate:
    cargo sqlx migrate run --source crates/rustvault-db/migrations

# Create a new migration (usage: just migrate-create my_migration)
migrate-create name:
    cargo sqlx migrate add -r {{ name }} --source crates/rustvault-db/migrations

# ── Docker ───────────────────────────────────────────────────

# Build Docker image
docker-build:
    docker build -f docker/Dockerfile -t rustvault:latest .

# Start all services via Docker Compose
docker-up:
    docker compose -f docker/docker-compose.yml up -d

# Stop all Docker Compose services
docker-down:
    docker compose -f docker/docker-compose.yml down

# Tail Docker Compose logs
docker-logs:
    docker compose -f docker/docker-compose.yml logs -f

# Start only the database container
docker-db:
    docker compose -f docker/docker-compose.yml up -d db

# ── Documentation ────────────────────────────────────────────

# Build mdBook documentation
docs:
    cd docs/book && mdbook build

# Serve mdBook documentation with live reload
docs-serve:
    cd docs/book && mdbook serve

# ── Cleanup ──────────────────────────────────────────────────

# Clean build artifacts
clean:
    cargo clean
    rm -rf web/dist web/node_modules
