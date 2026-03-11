# Testing Guide

This guide explains how to run and write tests for RustVault. See [ADR-0007](../docs/adr/0007-testing.md) for the full testing strategy rationale.

## Running Tests

### Prerequisites

- Rust 1.85+ with clippy and rustfmt (`rustup` default)
- A running PostgreSQL 16+ instance (or Docker)
- `DATABASE_URL` set in `.env` or environment

### Quick Commands

```bash
# Run all backend tests
just test

# Or manually:
cargo test --workspace

# Run only integration tests (requires a database)
cargo test -p rustvault-server --test api_tests

# Run a specific test by name
cargo test -p rustvault-server --test api_tests health_returns_ok

# Run with output (useful for debugging)
cargo test -p rustvault-server --test api_tests -- --nocapture
```

### Using `cargo-nextest` (Recommended)

[cargo-nextest](https://nexte.st/) runs tests in parallel and provides better output:

```bash
cargo install cargo-nextest --locked
cargo nextest run --workspace
```

## Test Architecture

### Testing Pyramid

```mermaid
%%{init: {"theme": "base", "themeVariables": {"fontSize": "14px"}}}%%
block-beta
    columns 1
    block:e2e:1
        E["E2E — Few, slow, high-confidence"]
    end
    block:int:1
        I["Integration — API + real database"]
    end
    block:unit:1
        U["Unit Tests — Fast, isolated, no I/O"]
    end
    style e2e fill:#f87171,color:#fff
    style int fill:#fbbf24,color:#000
    style unit fill:#34d399,color:#000
```

### Unit Tests

Located as inline `#[cfg(test)]` modules inside source files. These test pure functions — no database, no network.

**What to unit test:**
- Import format parsers
- Domain logic and calculations
- Validation rules
- Error mapping
- Serialization / deserialization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validates_email_format() {
        let result = validate_email("not-an-email");
        assert!(result.is_err());
    }
}
```

### Integration Tests

Located in `crates/rustvault-server/tests/`. These test API endpoints against a real PostgreSQL database.

**Key features:**
- **Ephemeral databases** — each test gets a fresh database via `#[sqlx::test]`
- **Automatic migrations** — `migrations = "../rustvault-db/migrations"` runs all SQL migrations
- **No TCP listener** — uses `axum-test` for in-process HTTP testing (fast, no port conflicts)

```rust
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn create_bank_succeeds(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let token = register_and_login(&server).await;
    
    let res = server
        .post("/api/banks")
        .authorization_bearer(&token)
        .json(&json!({"name": "Test Bank"}))
        .await;
        
    res.assert_status(StatusCode::CREATED);
    let body: Value = res.json();
    assert_eq!(body["data"]["name"], "Test Bank");
}
```

### Test Helpers

The `tests/helpers/mod.rs` module provides shared utilities:

| Helper | Purpose |
|--------|---------|
| `test_server(pool)` | Build an `axum-test` server with the given database pool |
| `register_and_login(server)` | Register a test user and return an access token |
| `TEST_USER` / `TEST_EMAIL` / `TEST_PASSWORD` | Default test credentials |
| `TEST_JWT_SECRET` | JWT secret used in all tests |

## Writing New Tests

### Adding a Unit Test

Add a `#[cfg(test)]` module at the bottom of the source file:

```rust
// In crates/rustvault-core/src/services/budget.rs

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn remaining_budget_subtracts_spent() {
        let budget = dec!(1000.00);
        let spent = dec!(350.75);
        assert_eq!(remaining(budget, spent), dec!(649.25));
    }
}
```

### Adding an Integration Test

Add a new test function in `crates/rustvault-server/tests/api_tests.rs`:

```rust
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn your_new_test(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let token = register_and_login(&server).await;
    
    // ... your test logic
}
```

The `#[sqlx::test]` macro:
1. Creates a temporary database
2. Runs all migrations from `rustvault-db/migrations/`
3. Passes the connection pool to your test function
4. Drops the temporary database when the test completes

### Testing Patterns

#### Full CRUD Lifecycle

Test create → list → get → update → delete in a single test to verify the complete flow:

```rust
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn entity_crud_lifecycle(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let token = register_and_login(&server).await;
    
    // Create
    let res = server.post("/api/entities")
        .authorization_bearer(&token)
        .json(&json!({"name": "Test"}))
        .await;
    res.assert_status(StatusCode::CREATED);
    let id = res.json::<Value>()["data"]["id"].as_str().unwrap().to_string();
    
    // List
    let res = server.get("/api/entities")
        .authorization_bearer(&token)
        .await;
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 1);
    
    // Get
    let res = server.get(&format!("/api/entities/{id}"))
        .authorization_bearer(&token)
        .await;
    res.assert_status_ok();
    
    // Update
    let res = server.put(&format!("/api/entities/{id}"))
        .authorization_bearer(&token)
        .json(&json!({"name": "Updated"}))
        .await;
    assert_eq!(res.json::<Value>()["data"]["name"], "Updated");
    
    // Delete
    let res = server.delete(&format!("/api/entities/{id}"))
        .authorization_bearer(&token)
        .await;
    res.assert_status(StatusCode::NO_CONTENT);
}
```

#### Data Isolation

Verify that users cannot access each other's data:

```rust
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn user_cannot_see_other_users_data(pool: sqlx::PgPool) {
    let server = test_server(pool);
    
    // User A creates data
    let token_a = register_and_login_as(&server, "a@test.com").await;
    server.post("/api/banks")
        .authorization_bearer(&token_a)
        .json(&json!({"name": "A's Bank"}))
        .await;
    
    // User B should see empty list
    let token_b = register_and_login_as(&server, "b@test.com").await;
    let res = server.get("/api/banks")
        .authorization_bearer(&token_b)
        .await;
    assert!(res.json::<Value>()["data"].as_array().unwrap().is_empty());
}
```

## Database Setup for Tests

Integration tests require `DATABASE_URL` to point to a PostgreSQL instance where `sqlx` can create temporary databases.

### Using Docker (Recommended)

```bash
# Start a test database
just docker-db

# Or manually:
docker run -d --name rustvault-test-db \
  -e POSTGRES_USER=rustvault \
  -e POSTGRES_PASSWORD=rustvault_dev \
  -e POSTGRES_DB=rustvault \
  -p 5432:5432 \
  postgres:17-alpine
```

Set in `.env`:
```bash
DATABASE_URL=postgres://rustvault:rustvault_dev@localhost:5432/rustvault
```

### CI

CI uses a PostgreSQL service container. See `.github/workflows/` for the exact configuration.

## Coverage

Generate a code coverage report with [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov):

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --html
open target/llvm-cov/html/index.html
```
