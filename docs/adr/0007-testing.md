# ADR-0007: Application Testing

- **Status:** Accepted
- **Date:** 2026-03-03
- **Deciders:** RustVault core team
- **Tags:** testing, backend, frontend, ci, quality

## Context

RustVault handles sensitive financial data — incorrect calculations, broken imports, or silent data loss are unacceptable. The testing strategy must:

- Catch regressions before they reach users
- Cover the full stack — Rust backend, SolidJS frontend, API integration, and import pipeline
- Run fast enough to encourage frequent execution during development
- Integrate seamlessly with CI/CD
- Validate financial correctness (decimal precision, currency conversion, rounding)

## Decisions

### Testing Pyramid

```
         ╱ E2E ╲              Few, slow, high-confidence
        ╱───────╲
       ╱ Integration ╲        Moderate count, API + DB
      ╱─────────────────╲
     ╱    Unit Tests      ╲   Many, fast, isolated
    ╱───────────────────────╲
```

The majority of tests are fast unit tests. Integration tests validate API + database interactions. E2E tests cover critical user flows only.

### Backend Testing (Rust)

#### Unit Tests

- **Location:** Inline `#[cfg(test)]` modules in each source file
- **Runner:** `cargo nextest` (parallel, faster than `cargo test`)
- **Scope:** Pure functions, domain logic, parsers, validators, error mapping
- **Key areas:**
  - Import format parsers (CSV, MT940, OFX, QIF, CAMT.053, XLSX, JSON)
  - Auto-categorization rule matching
  - Budget calculations and forecasting
  - Currency conversion and decimal precision
  - Auth token generation and validation
  - Audit log serialization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_currency_conversion_precision() {
        let amount = dec!(100.50);
        let rate = dec!(1.0856);
        let result = convert_currency(amount, rate);
        assert_eq!(result, dec!(109.1028));
    }
}
```

#### Integration Tests

- **Location:** `tests/` directory in each crate
- **Database:** Ephemeral PostgreSQL per test via `sqlx::test` (creates temp DB, runs migrations, drops after)
- **Scope:** API endpoint testing with real database
- **Key areas:**
  - Full CRUD lifecycle for each entity (create → read → update → delete)
  - Auth flow (register → login → JWT → access protected route)
  - Import pipeline (upload file → parse → map columns → confirm → verify transactions)
  - Multi-user data isolation (user A cannot see user B's data)
  - Pagination and filtering
  - Concurrent access patterns

```rust
#[sqlx::test(migrations = "migrations")]
async fn test_import_csv_creates_transactions(pool: PgPool) {
    let app = test_app(pool).await;
    let token = create_test_user_and_login(&app).await;
    
    let csv = "date,amount,description\n2026-01-15,-42.50,Grocery Store\n";
    let response = app.import_csv(&token, csv).await;
    
    assert_eq!(response.status(), 200);
    assert_eq!(response.json().imported_count, 1);
}
```

#### Property-Based Tests

- **Crate:** `proptest`
- **Scope:** Import parsers, currency math, rule matching
- Generate random but valid inputs to catch edge cases that hand-written tests miss

### Frontend Testing (SolidJS)

#### Unit Tests

- **Runner:** `vitest` (Vite-native, fast)
- **Scope:** Utility functions, store logic, formatters, validators
- **Key areas:**
  - Currency formatting per locale
  - Date formatting and timezone handling
  - Form validation logic
  - Filter/sort transformations
  - i18n key resolution

#### Component Tests

- **Runner:** `vitest` + `@solidjs/testing-library`
- **Scope:** Individual components in isolation
- **Key areas:**
  - Transaction row rendering
  - Form submission and validation feedback
  - Error boundary behavior
  - Accessible markup (ARIA attributes, keyboard navigation)
  - Responsive layout breakpoints

```typescript
import { render, screen } from "@solidjs/testing-library";
import { TransactionRow } from "./TransactionRow";

test("displays formatted amount with currency symbol", () => {
  render(() => <TransactionRow amount={-42.50} currency="EUR" locale="de-DE" />);
  expect(screen.getByText("-42,50 €")).toBeInTheDocument();
});
```

#### Visual Regression Tests

- **Tool:** Storybook + Chromatic (or Percy)
- **Scope:** Component catalog snapshots — catch unintended style changes
- Run on PRs; baseline updated on merge to main

### API Contract Testing

- **OpenAPI validation:** Every API response is validated against the `utoipa`-generated OpenAPI spec in integration tests
- **Schema drift detection:** CI fails if the generated `openapi.json` differs from the committed spec (ensures docs stay in sync)

### Import Pipeline Testing

Import parsing is the most critical path. Testing strategy:

| Test type | What it covers |
|-----------|----------------|
| **Parser unit tests** | Each format parser with known-good fixture files |
| **Fixture files** | Real-world bank exports (anonymized) in `tests/fixtures/` |
| **Edge cases** | Empty files, malformed rows, mixed encodings, BOM markers, dates in ambiguous formats |
| **Round-trip tests** | Import → export → re-import produces identical data |
| **Fuzz testing** | `cargo-fuzz` on parsers to catch panics on malformed input |

### E2E Tests

- **Runner:** Playwright
- **Scope:** Critical user flows only — not exhaustive UI coverage
- **Key flows:**
  1. Sign up → first login → dashboard renders
  2. Import CSV → map columns → confirm → transactions appear
  3. Create budget → assign categories → verify calculations
  4. Multi-user: create household member → verify role permissions
- **Environment:** Docker Compose (app + PostgreSQL) running in CI

### Performance Testing

- **Lighthouse CI:** Enforces performance budget (FCP <1s, TTI <1.5s, bundle <150KB)
- **k6 load tests:** API endpoint response times under concurrent load (target: p95 <200ms for reads)
- **Import benchmark:** Measure parse time for 10k-row CSV, MT940, and OFX files — fail CI if regression >10%

### Test Data

- **Factories:** `test_helpers` crate with builder pattern for creating test users, accounts, transactions
- **Fixtures:** Anonymized real bank exports for each supported format in `tests/fixtures/`
- **Seeds:** Optional dev seed script for local development with realistic data volume

### CI Integration

```
PR opened / push to main
    │
    ├── cargo nextest (unit + integration)     ~2 min
    ├── cargo clippy (lints)                   ~1 min
    ├── cargo fmt --check                      ~10s
    ├── sqlx prepare --check (offline mode)    ~30s
    ├── vitest (frontend unit + component)     ~1 min
    ├── OpenAPI spec drift check               ~10s
    ├── Playwright E2E (Docker Compose)        ~3 min
    ├── Lighthouse CI                          ~1 min
    └── cargo-deny (license + advisory audit)  ~30s
```

Total CI time target: **< 10 minutes**.

### Coverage

- **Backend:** `cargo-llvm-cov` — target 80%+ line coverage on core crates (domain logic, parsers, API handlers)
- **Frontend:** `vitest --coverage` — target 70%+ on utility and store logic
- Coverage reports published as CI artifacts; no hard gate (coverage is a signal, not a target)

## Consequences

### Positive

- `cargo nextest` runs tests in parallel — fast feedback loop
- `sqlx::test` ephemeral databases ensure clean state without manual teardown
- Property-based and fuzz testing catch edge cases that hand-written tests miss
- OpenAPI contract testing guarantees docs match implementation
- Performance regression detection prevents silent degradation
- Import fixture tests validate real-world bank file formats

### Negative

- Integration tests require PostgreSQL — local Docker or CI service container needed
- Playwright E2E tests are slower and more brittle than unit tests
- Maintaining fixture files adds ongoing effort as banks change export formats
- Fuzz testing requires dedicated CI time and doesn't always find issues quickly

### Risks

- Test suite becomes slow as codebase grows (mitigated: `nextest` parallelism, test categorization with `#[ignore]` for slow tests)
- Flaky E2E tests undermine CI trust (mitigated: retry logic, minimal E2E scope, stable selectors)

## References

- [cargo-nextest](https://nexte.st/)
- [sqlx::test](https://docs.rs/sqlx/latest/sqlx/attr.test.html)
- [proptest](https://github.com/proptest-rs/proptest)
- [Vitest](https://vitest.dev/)
- [Solid Testing Library](https://github.com/solidjs/solid-testing-library)
- [Playwright](https://playwright.dev/)
- [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
