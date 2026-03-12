# ADR-0010: Dual I18n — Fluent (Backend) + ICU-style JSON (Frontend)

- **Status:** Accepted
- **Date:** 2026-03-12
- **Deciders:** Core team
- **Tags:** frontend, backend, i18n

## Context

RustVault targets a global audience and must display all user-facing
strings — error messages, labels, validation hints — in the user's
preferred language.  The project has two codebases with fundamentally
different runtimes:

- **Rust backend** — Axum server, no JS runtime, needs concurrent bundle
  access from multiple request threads.
- **SolidJS frontend** — client-side SPA, needs reactive re-renders when
  the locale changes, tiny bundle size, and lazy loading.

We need an i18n architecture that:

1. Supports pluralisation, gender, and template parameters.
2. Allows community-contributed translations without code changes.
3. Keeps frontend bundles small (only load the active locale).
4. Provides compile-time key safety wherever possible.
5. Works well with both runtimes and their respective ecosystems.

## Decision

Use **two complementary i18n systems**, one per runtime:

### Backend — Project Fluent

- Library: `fluent-bundle` (Rust), `fluent-langneg` for negotiation.
- Format: `.ftl` files in `locales/{locale}/`.
- Loaded once at server startup into `Arc<FluentBundle>` per locale.
- Thread-safe via `FluentBundle<FluentResource, IntlLangMemoizer>`.
- Accept-Language header parsed per request; best locale chosen via
  `fluent_langneg::negotiate_languages`.
- Messages support Fluent's full syntax: selectors, plurals, terms.

### Frontend — @solid-primitives/i18n with JSON

- Library: `@solid-primitives/i18n` (40 LOC, tree-shakeable).
- Format: nested JSON files in `web/src/locales/{locale}/`, one file per
  namespace (common, auth, banks, …).
- Loaded lazily via Vite dynamic imports — only the active locale's
  dictionaries are fetched.
- `flatten()` converts nested JSON to dot-separated keys.
- `translator()` returns a reactive `t()` function that re-renders
  components when the locale signal changes.
- `resolveTemplate` handles `{{param}}` interpolation.
- TypeScript's `typeof import()` derives key types from the en-US JSON
  files, giving **compile-time key safety** with zero code generation.

### Shared Conventions

- Both layers use **BCP 47** locale codes (`en-US`, `de-DE`, `pl-PL`).
- Locale metadata (display name, native name, completeness) stored in
  `locales/_meta.toml`.
- New locales are added by copying the `en-US` directory, translating
  values, and updating one type union + one array constant.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| **react-i18next / i18next** | Massive ecosystem, ICU MessageFormat plugin, namespaces | ~45 kB bundle; React-centric API; unneeded complexity for SolidJS |
| **Fluent everywhere** (incl. frontend) | Single format across full stack | No mature SolidJS binding; `.ftl` parsing adds ~20 kB to the browser bundle; IDE support is weaker for frontenders |
| **FormatJS / @formatjs/intl** | Full ICU MessageFormat; React bindings | React-oriented; heavy runtime; no first-party SolidJS support |
| **Paraglide.js (Inlang)** | Compile-time, tree-shaken per-message | Young ecosystem; no Fluent backend story; requires inlang project config |
| **typesafe-i18n** | Zero-runtime, type-safe generation | Requires code-gen step in CI; no SolidJS adapter; adds build complexity |

## Consequences

### Positive

- **Tiny frontend bundle** — `@solid-primitives/i18n` is < 1 kB gzipped
  and the JSON dictionaries are lazy-loaded per locale.
- **Full compile-time safety** on the frontend — TypeScript catches
  missing or misspelled keys at `tsc` time with no code generation.
- **Fluent's power on the backend** — plurals, selectors, terms, and
  message references cover complex server-side formatting needs.
- **Easy to contribute translations** — copy a folder, translate JSON
  values, run `npm run typecheck`.
- **Independent evolution** — frontend and backend i18n can be upgraded
  independently.

### Negative

- **Two formats to learn** — contributors must know JSON key conventions
  (frontend) and Fluent syntax (backend).
- **Key parity not enforced cross-stack** — a backend `.ftl` key like
  `auth-invalid-credentials` has no automated link to the frontend's
  `auth.errors.invalidCredentials`.

### Risks

- **Stale translations** — without CI completeness checks, non-default
  locales can fall behind `en-US` silently.  Mitigated by the backend's
  built-in completeness percentage and a planned CI lint step.
- **@solid-primitives/i18n churn** — the library is community-maintained.
  Mitigated by its tiny API surface (< 200 LOC); easy to vendor if
  abandoned.

## References

- [Project Fluent — fluent.rs](https://github.com/projectfluent/fluent-rs)
- [@solid-primitives/i18n](https://github.com/solidjs-community/solid-primitives/tree/main/packages/i18n)
- [BCP 47 — Language Tags](https://www.rfc-editor.org/info/bcp47)
- [ADR-0006: Error Handling](0006-error-handling.md)
- [Translation Guide](../contributing/translation-guide.md)
