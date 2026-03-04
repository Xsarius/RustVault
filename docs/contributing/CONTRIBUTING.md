# Contributing to RustVault

Thank you for your interest in contributing to RustVault! This guide will help you get started.

## Development Environment Setup

### Prerequisites

- **Rust** (stable, >= 1.85) — install via [rustup](https://rustup.rs/)
- **Node.js** (>= 22) or **Bun** (>= 1.0)
- **PostgreSQL** (>= 16) — or use Docker
- **Docker** & **Docker Compose** — for the full stack
- **[just](https://just.systems)** — command runner

### Getting Started

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Xsarius/RustVault.git
   cd RustVault
   ```

2. **Copy environment variables:**
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials and JWT secret
   ```

3. **Start the database:**
   ```bash
   just docker-db
   ```

4. **Run the backend:**
   ```bash
   cargo run -p rustvault-server
   ```

5. **Run the frontend:**
   ```bash
   cd web && bun install && bun run dev
   ```

6. **Or run both together:**
   ```bash
   just dev
   ```

### Useful Commands

| Command | Description |
|---------|-------------|
| `just dev` | Start backend + frontend in parallel |
| `just test` | Run all tests |
| `just lint` | Run clippy + frontend lint |
| `just fmt` | Format all code |
| `just docker-up` | Start full stack via Docker Compose |
| `just` | Show all available commands |

## Project Structure

```
crates/
├── rustvault-server/    # Binary — Axum HTTP server
├── rustvault-core/      # Library — domain logic, services
├── rustvault-db/        # Library — SQLx queries, migrations
├── rustvault-import/    # Library — file parsers & import engine
└── rustvault-ai/        # Library — AI features (optional)
web/                     # SolidJS frontend
docker/                  # Dockerfile & docker-compose.yml
docs/                    # Documentation (mdBook, ADRs, plans)
locales/                 # Backend i18n files (Project Fluent)
```

## Code Style

### Rust

- Follow standard Rust conventions (`cargo fmt`, `cargo clippy`).
- All public items must have `///` doc comments.
- Use `thiserror` for error types in library crates.
- Use `anyhow` only in `main.rs` and test code.
- Financial amounts use `rust_decimal::Decimal`, never `f64`.

### TypeScript

- Use TypeScript strict mode.
- Prefer SolidJS idioms (signals, resources, stores).
- All UI strings must use the i18n system — no hardcoded strings.

## Pull Request Process

1. Fork the repo and create a feature branch from `main`.
2. Write tests for new functionality.
3. Ensure `just lint` and `just test` pass.
4. Update documentation if you're changing public APIs.
5. Open a PR with a clear description of the changes.

## Architecture Decision Records

Significant technical decisions are documented as ADRs in `docs/adr/`. When proposing a change to the architecture, include a new ADR with your PR.

## License

By contributing, you agree that your contributions will be licensed under the AGPL-3.0 license.
