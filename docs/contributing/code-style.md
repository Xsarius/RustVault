# Code Style Guide

This document describes the coding conventions used across the RustVault codebase. All contributors should follow these guidelines to maintain consistency.

## Tooling

All style enforcement is automated. Run these before committing:

```bash
just fmt    # Format all code (Rust + TypeScript)
just lint   # Run clippy + frontend linter
```

CI blocks merges that fail formatting or linting checks.

## Rust

### Formatting (`rustfmt.toml`)

| Setting | Value | Notes |
|---------|-------|-------|
| `edition` | `2024` | Rust 2024 edition formatting rules |
| `max_width` | `100` | Line length limit |
| `tab_spaces` | `4` | Indent with 4 spaces |
| `use_field_init_shorthand` | `true` | `Foo { x }` instead of `Foo { x: x }` |
| `use_try_shorthand` | `true` | `x?` instead of `try!(x)` |

Run `cargo fmt --all` to format. The formatter is authoritative — do not fight it.

### Linting (`clippy.toml`)

| Setting | Value |
|---------|-------|
| `msrv` | `1.85` |

We use the default clippy lint set plus `#![warn(missing_docs)]` in library crates. All public items must have `///` doc comments.

### Error Handling

- **Library crates** (`rustvault-core`, `rustvault-db`, `rustvault-import`, `rustvault-ai`): use `thiserror` with a crate-level error enum (e.g., `CoreError`, `DbError`).
- **Binary crate** (`rustvault-server`): convert domain errors to HTTP responses via `ApiError`. Use `anyhow` only in `main.rs` and test helpers.
- Never use `.unwrap()` in library code. Acceptable in tests and `main.rs` after configuration loading.

### Numeric Types

- **Financial amounts:** always `rust_decimal::Decimal`. Never use `f32` or `f64` for money.
- **IDs:** `uuid::Uuid` (v7 for new records — time-sortable).
- **Timestamps:** `time::OffsetDateTime` (UTC).

### Module Organisation

```
crate/src/
├── lib.rs          # Re-exports, #![warn(missing_docs)]
├── error.rs        # Crate-level error enum
├── models/         # Data structures (DTOs, domain types)
│   └── mod.rs      # Re-exports all model types
└── services/       # Business logic functions
    └── mod.rs      # Re-exports all service modules
```

- One file per domain entity (e.g., `models/bank.rs`, `services/bank.rs`).
- `mod.rs` files use `pub mod` + `pub use` for a flat public API.

### Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Types / Structs | `PascalCase` | `NewBank`, `UserInfo` |
| Functions / Methods | `snake_case` | `create_bank`, `list_by_user` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_PAGE_SIZE` |
| Module names | `snake_case` | `models::bank`, `services::auth` |
| CRUD functions | `list`, `create`, `get`, `update`, `delete` | Consistent across all entities |

### Import Order

Group imports in this order, separated by blank lines:

1. Standard library (`std::*`)
2. Third-party crates (`axum`, `serde`, `sqlx`, etc.)
3. Workspace crates (`rustvault_core`, `rustvault_db`)
4. Crate-internal (`crate::`, `super::`)

`rustfmt` handles the sorting within each group.

### SQL Queries

- All database queries live in `rustvault-db/src/repos/`.
- Use `sqlx::query!` / `sqlx::query_as!` macros for compile-time checked queries where possible.
- Raw `sqlx::query_scalar` is acceptable for simple queries (e.g., `SELECT 1`).
- Parameter names use snake_case and match the Rust struct field names.

### Documentation Comments

```rust
/// Short one-line summary.
///
/// Longer description if needed. Use backticks for `code references`.
/// Explain non-obvious parameters and return values.
pub fn create(pool: &PgPool, user_id: Uuid, name: &str) -> Result<Bank, CoreError> {
    // ...
}
```

- All `pub` items **must** have `///` doc comments.
- Module-level docs use `//!` at the top of the file.
- Write in third person imperative: "Creates a bank" not "This creates a bank".

## TypeScript (Frontend)

### General

- TypeScript strict mode (`strict: true` in `tsconfig.json`)
- No `any` types — use `unknown` and narrow
- Prefer `const` over `let`

### SolidJS Patterns

- Use **signals** for local component state
- Use **resources** for async data fetching
- Use **stores** for complex nested state
- Components are functions returning JSX — no classes

### Internationalisation

- All user-facing strings **must** use the i18n system
- No hardcoded strings in components
- Translation keys follow the pattern `section.key` (e.g., `auth.login_button`)

### File Naming

| Item | Convention | Example |
|------|-----------|---------|
| Components | `PascalCase.tsx` | `TransactionList.tsx` |
| Utilities | `camelCase.ts` | `formatCurrency.ts` |
| Styles | `component.css` | `transactionList.css` |
| Tests | `*.test.ts` / `*.test.tsx` | `formatCurrency.test.ts` |

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): short description

Optional longer body explaining the change.
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `ci`.

Examples:
```
feat(banks): add archive endpoint
fix(categories): cast enum to TEXT in Postgres query
docs(adr): add OIDC integration design record
test(integration): add tag CRUD test suite
```
