# ADR-0001: Backend Architecture

- **Status:** Accepted
- **Date:** 2026-03-03
- **Deciders:** RustVault core team
- **Tags:** backend, rust, axum, architecture

## Context

RustVault is a self-hosted personal finance platform handling sensitive financial data. The backend must:

- Parse thousands of transactions from multiple formats (CSV, MT940, OFX, CAMT.053, XLSX, QIF, JSON) in seconds
- Run unattended on home servers for years without memory leaks or degradation
- Deploy as a single artifact — no runtime dependencies for self-hosters
- Guarantee memory safety and type safety for financial operations
- Support real-time features (import progress via WebSocket)

## Decisions

### Language: Rust

Rust's ownership model eliminates data races and memory bugs at compile time. Its type system enforces correctness for monetary calculations. The compiled binary deploys as a single Docker artifact.

| Alternative | Why not |
|-------------|---------|
| **Go** | No algebraic types; GC pauses; less expressive error handling |
| **TypeScript/Node.js** | Runtime-only type safety; floating-point precision issues for money; larger deployment |
| **Python** | Slow for parsing workloads; GIL limits concurrency; complex deployment |
| **Java/Kotlin** | JVM overhead for self-hosting; large memory footprint; slower startup |

### HTTP Framework: Axum

Axum is built on Tokio, Hyper, and Tower — providing native async I/O, Tower middleware compatibility, and type-safe request extractors.

| Alternative | Why not |
|-------------|---------|
| **Actix-web** | Actor model adds complexity; not Tower-native |
| **Rocket** | Heavier macro magic; slower async adoption |
| **Warp** | Complex type errors; less intuitive for large APIs |

### Runtime: Tokio

Single async runtime across HTTP, database, file I/O, and WebSocket connections. No mixed-runtime overhead.

### Crate Architecture

```
rustvault/
├── rustvault-server      # Axum HTTP server, main binary
├── rustvault-core        # Domain types, business logic, auth (JWT, OIDC, argon2), shared utilities
├── rustvault-db          # SQLx queries, migrations
├── rustvault-import      # Format parsers (CSV, MT940, OFX, etc.)
└── rustvault-ai          # Optional AI features (toggleable)
```

Auth (JWT, OIDC, password hashing) and common utilities (error types, shared types) live in `rustvault-core` to avoid excessive crate fragmentation while maintaining clear module boundaries within the crate.

Each crate compiles independently — enabling parallel builds, focused tests, and clear dependency boundaries.

## Consequences

### Positive

- Memory safety without GC — no null pointers, buffer overflows, or use-after-free
- Fearless concurrency — parallel import parsing without data races
- Single binary — `docker run rustvault` just works
- Near-C speed for file parsing; sub-second imports of large bank statements
- Tower middleware ecosystem — rate limiting, timeouts, CORS, logging out of the box
- WebSocket support via `axum::extract::ws` for real-time import progress
- Type-safe extractors (`Json<T>`, `Path<T>`, `Query<T>`) catch errors at compile time

### Negative

- Steeper learning curve (ownership, lifetimes, borrow checker)
- Slower compilation times compared to Go
- Smaller contributor pool than JS/Python ecosystems
- Tower service trait adds complexity for custom middleware

### Risks

- Compile times grow with codebase (mitigated: workspace splitting, incremental compilation, `cargo-nextest`)
- Axum is pre-1.0 (mitigated: locked versions, active Tokio team maintenance)

## References

- [Rust language](https://www.rust-lang.org/)
- [Axum](https://github.com/tokio-rs/axum)
- [Tower middleware](https://github.com/tower-rs/tower)
- [Tokio async runtime](https://tokio.rs/)
