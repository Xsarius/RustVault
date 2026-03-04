# ADR-0004: Data Model & Persistence

- **Status:** Accepted
- **Date:** 2026-03-03
- **Deciders:** RustVault core team
- **Tags:** database, postgresql, sqlx, data-model

## Context

RustVault manages financial data with complex relationships: users → banks → accounts → transactions, plus categories, tags, budgets, import history, and audit logs. The persistence layer must:

- Support multi-user with strict data isolation
- Handle flexible metadata (bank-specific fields, import source details)
- Enable full-text search across transaction descriptions
- Perform well with 100k+ transactions per user
- Provide compile-time query safety
- Keep migrations versioned and embedded in the binary

## Decisions

### Database: PostgreSQL

PostgreSQL provides multi-user support, JSONB for flexible metadata, robust full-text search (`tsvector`), and advanced query features (CTEs, window functions) needed for financial reporting.

| Alternative | Why not |
|-------------|---------|
| **SQLite** | Single-writer limits multi-user; no concurrent write access; harder to scale |
| **MySQL/MariaDB** | Weaker JSONB support; less expressive window functions; fewer advanced features |
| **MongoDB** | Schema-less adds risk for financial data; no transactional guarantees across documents |

### Query Layer: SQLx (compile-time checked SQL)

SQLx provides compile-time verified raw SQL queries, native async (Tokio), and direct access to PostgreSQL features without ORM abstraction.

| Alternative | Why not |
|-------------|---------|
| **Diesel** | Sync-only (requires `spawn_blocking`); heavy DSL; struggles with JSONB |
| **SeaORM** | Extra abstraction; query builder obscures complex SQL; runtime-checked |
| **Raw tokio-postgres** | No compile-time checks; manual type mapping; no migration tooling |

`sqlx::query!()` checks queries against a live database at compile time. Offline mode via `sqlx prepare` generates metadata for CI builds without a running database.

### Migrations: SQLx migrate

Embedded in the binary — no external tool needed. Versioned SQL files in `migrations/`. Each migration file has a header comment explaining the schema change.

### Core Entity Model

```
User ──┬── Bank ──── Account ──── Transaction ──┬── Tag (M:N)
       │                                        └── Transfer (pair)
       ├── Category (tree)
       ├── AutoRule
       ├── Budget ──── BudgetLine
       ├── Import (history)
       └── AuditLog
```

### Key Schema Decisions

| Decision | Rationale |
|----------|-----------|
| **UUIDs as primary keys** | No sequential enumeration; safe for external APIs; merge-friendly |
| **JSONB `metadata` columns** | Bank-specific fields, import source details — flexible without schema changes |
| **Soft deletes** | Transactions use `deleted_at`; financial history should never be lost |
| **Hierarchical categories** | `parent_id` self-reference enables subcategories (e.g., Food → Groceries, Restaurants) |
| **`tsvector` for search** | Full-text search across transaction descriptions, payees, notes |
| **Decimal for money** | `NUMERIC(19,4)` — no floating-point precision issues |
| **Audit log** | Every mutation writes to `audit_log` with `entity_type`, `entity_id`, `action`, `old_value`, `new_value` |
| **Dual auth model** | `auth_provider` field (`local`/`oidc`/`both`); `password_hash` nullable for OIDC-only users |

### Data Isolation

All queries filter by `user_id`. Row-level security is enforced at the query layer — no cross-user data leakage. Multi-user households share data via explicit roles, not implicit access.

## Consequences

### Positive

- Compile-time verified SQL catches typos, type mismatches, and schema drift before runtime
- Full PostgreSQL power — JSONB, CTEs, window functions, `tsvector` used directly
- Built-in migrations embedded in binary — no external tooling
- NUMERIC(19,4) guarantees precise monetary calculations
- Audit log provides full edit history for compliance and debugging

### Negative

- Raw SQL requires discipline (mitigated: parameterized queries enforced by `sqlx::query!`)
- No automatic relation loading — joins written manually
- Compile-time checking needs a running database (mitigated: `sqlx prepare` for offline)

### Risks

- Large query surfaces can be hard to maintain (mitigated: organized query modules per domain entity)
- JSONB queries are slower than typed columns for frequent filters (mitigated: typed columns for common fields, JSONB for rare metadata)

## References

- [SQLx](https://github.com/launchbadge/sqlx)
- [PostgreSQL JSONB](https://www.postgresql.org/docs/current/datatype-json.html)
- [PostgreSQL full-text search](https://www.postgresql.org/docs/current/textsearch.html)
