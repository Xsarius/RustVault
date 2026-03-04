# ADR-0006: Error Handling

- **Status:** Accepted
- **Date:** 2026-03-03
- **Deciders:** RustVault core team
- **Tags:** errors, backend, frontend, api

## Context

A finance application must handle errors gracefully at every layer — from database constraint violations to network timeouts to invalid import files. Users need clear, actionable feedback. Developers need structured, debuggable error information. The error strategy must:

- Provide consistent error responses across all API endpoints
- Never leak internal details (SQL queries, stack traces) to clients
- Support localized error messages for international users
- Make error paths explicit and composable in Rust
- Give the frontend enough structure to render contextual error UIs

## Decisions

### Backend: `Result<T, AppError>` everywhere

All fallible operations return `Result<T, AppError>`. The `AppError` enum covers every domain error category:

```rust
pub enum AppError {
    // Auth
    Unauthorized,                    // 401 — missing or invalid token
    Forbidden,                       // 403 — valid token, insufficient role
    
    // Validation
    Validation(Vec<FieldError>),     // 422 — structured field-level errors
    
    // Domain
    NotFound(EntityType, Uuid),      // 404 — entity not found
    Conflict(String),                // 409 — duplicate import, unique violation
    BusinessRule(String),            // 400 — domain logic rejection
    
    // Import
    ImportParse(ImportError),        // 422 — file parsing failure with row/column
    ImportUnsupported(String),       // 415 — unsupported file format
    
    // Infrastructure
    Database(sqlx::Error),           // 500 — mapped, never exposed raw
    Internal(anyhow::Error),         // 500 — catch-all, logged, generic response
    
    // Rate limiting
    RateLimited(Duration),           // 429 — includes retry-after
}
```

`AppError` implements `IntoResponse` (Axum trait) — each variant maps to an HTTP status code and structured JSON body. Internal errors are logged with full context but return a generic message to the client.

### API Error Response Format

Every error response follows the same envelope:

```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Some fields are invalid.",
    "details": [
      { "field": "amount", "code": "REQUIRED", "message": "Amount is required." },
      { "field": "date", "code": "INVALID_FORMAT", "message": "Date must be YYYY-MM-DD." }
    ]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `code` | `string` | Machine-readable error code (e.g., `VALIDATION_FAILED`, `NOT_FOUND`, `UNAUTHORIZED`) |
| `message` | `string` | Human-readable, localized message (uses Fluent with user's locale) |
| `details` | `array?` | Optional field-level errors for validation failures |

### Error Codes

Stable, documented error codes that clients can match on:

| Code | HTTP | When |
|------|------|------|
| `UNAUTHORIZED` | 401 | Missing/invalid/expired JWT |
| `FORBIDDEN` | 403 | Valid JWT, insufficient permissions |
| `NOT_FOUND` | 404 | Entity doesn't exist or belongs to another user |
| `VALIDATION_FAILED` | 422 | Request body fails schema or business validation |
| `CONFLICT` | 409 | Duplicate import, unique constraint violation |
| `IMPORT_PARSE_ERROR` | 422 | File parsing failed (with row/column details) |
| `UNSUPPORTED_FORMAT` | 415 | Import file format not recognized |
| `RATE_LIMITED` | 429 | Too many requests (includes `Retry-After` header) |
| `INTERNAL_ERROR` | 500 | Unexpected failure (generic message, details logged server-side) |

### Localized Error Messages

Error messages are localized using Fluent (backend i18n). The user's locale is resolved from `Accept-Language` header or `user.locale` setting. Error keys map to `.ftl` locale files:

```ftl
# errors.ftl
error-not-found = { $entity_type } not found.
error-validation-required = { $field } is required.
error-import-parse = Failed to parse row { $row }: { $reason }.
```

### Frontend Error Handling

The SolidJS frontend handles errors at three levels:

1. **API client layer** — catches HTTP errors, maps `error.code` to UI behavior (redirect to login on 401, show toast on 500, inline field errors on 422)
2. **Component layer** — `ErrorBoundary` components catch rendering errors with fallback UIs
3. **Form layer** — `@modular-forms/solid` handles field-level validation with immediate feedback

Toast notifications for transient errors (network issues, rate limiting). Inline messages for persistent errors (validation, not found).

### Import Error Handling

Import errors are special — a single file may have hundreds of rows, some valid and some not. Strategy:

- Parse errors are collected per row, not thrown immediately
- Users see a summary: "247 of 250 transactions imported. 3 rows skipped."
- Skipped rows are displayed with row number, raw data, and reason
- Users can fix and re-import only the failed rows

### Logging

- Structured logging via `tracing` crate (JSON format in production)
- Every error log includes: request ID, user ID, error variant, context
- 5xx errors log full stack trace; 4xx errors log at `warn` level
- No sensitive data (passwords, tokens, PII) in log output

## Consequences

### Positive

- Consistent error envelope — frontend never needs to guess response shape
- Machine-readable codes enable client-side error routing without parsing messages
- Localized messages — users see errors in their language
- Explicit `Result<T, AppError>` — no forgotten error paths; compiler enforces handling
- Structured import errors let users fix and retry without re-importing everything
- `tracing` gives structured, searchable logs with request correlation

### Negative

- AppError enum grows with the domain — needs periodic review to avoid catch-all variants
- Fluent error key sync adds maintenance overhead
- Comprehensive error handling adds code volume to every handler

### Risks

- Error codes become an implicit API contract — changing them is a breaking change (mitigated: document codes in OpenAPI spec, version if needed)

## References

- [RFC 7807 — Problem Details for HTTP APIs](https://datatracker.ietf.org/doc/html/rfc7807)
- [Rust error handling patterns](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [tracing crate](https://github.com/tokio-rs/tracing)
- [Axum error handling](https://docs.rs/axum/latest/axum/error_handling/index.html)
