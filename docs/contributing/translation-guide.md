# Translation Guide

This document explains how the RustVault frontend i18n system works and
how to add a new locale.

---

## Architecture Overview

RustVault uses **two separate i18n systems**:

| Layer | Library | Format | Directory |
|-------|---------|--------|-----------|
| Backend (Rust) | [Fluent](https://projectfluent.org/) | `.ftl` files | `locales/{locale}/*.ftl` |
| Frontend (SolidJS) | [@solid-primitives/i18n](https://github.com/solidjs-community/solid-primitives/tree/main/packages/i18n) | `.json` files | `web/src/locales/{locale}/*.json` |

Both systems share the same BCP 47 locale codes (e.g. `en-US`, `de-DE`,
`pl-PL`).

---

## Frontend Locale Files

Locale files live under `web/src/locales/{locale}/` and are split into
**namespaces** — one JSON file per domain area:

```
web/src/locales/
└── en-US/
    ├── common.json       # Navigation, actions, generic labels
    ├── auth.json          # Login, register, password reset
    ├── banks.json         # Banks & accounts screens
    ├── categories.json    # Category management
    ├── tags.json          # Tag management
    └── settings.json      # Settings page
```

### Key Naming Conventions

Keys use **nested JSON objects** that are flattened to dot-separated
paths at runtime (e.g. `nav.dashboard`).

```json
{
  "nav": {
    "dashboard": "Dashboard",
    "transactions": "Transactions"
  },
  "actions": {
    "save": "Save",
    "cancel": "Cancel",
    "delete": "Delete"
  }
}
```

Conventions:

- **camelCase** keys (`accountType`, not `account_type`).
- **Group by feature** — keep related keys together under a common
  parent object.
- **Keep values short** — translate UI labels, not paragraphs.
- **Avoid embedding HTML** — use template parameters instead.

### Template Parameters

The `@solid-primitives/i18n` template resolver supports `{{param}}`
placeholders:

```json
{
  "greeting": "Welcome back, {{name}}!"
}
```

```tsx
t("common.greeting", { name: user().displayName });
```

---

## Adding a New Locale

### Step 1 — Create the directory

Copy the `en-US` folder as a template:

```bash
cp -r web/src/locales/en-US web/src/locales/de-DE
```

### Step 2 — Translate the JSON files

Open each file in the new directory and translate every **value**
(never change the keys):

```json
{
  "nav": {
    "dashboard": "Übersicht",
    "transactions": "Transaktionen"
  }
}
```

### Step 3 — Register the locale

Open `web/src/i18n/index.tsx` and make three changes:

#### 3a. Extend the `Locale` type

```diff
- export type Locale = "en-US";
+ export type Locale = "en-US" | "de-DE";
```

#### 3b. Add the type import for compile-time key checking

Add the new locale's common namespace to the `RawDictionary` type or
verify the new JSON files match the same structure as `en-US`.

#### 3c. Add the locale to `SUPPORTED_LOCALES`

```diff
- export const SUPPORTED_LOCALES: readonly Locale[] = ["en-US"] as const;
+ export const SUPPORTED_LOCALES: readonly Locale[] = ["en-US", "de-DE"] as const;
```

The `fetchDictionary` function already uses dynamic imports with the
locale code, so no further loader changes are needed — it will
automatically discover `web/src/locales/de-DE/*.json`.

### Step 4 — Add a new namespace (optional)

If you need a new namespace (e.g. `import.json`):

1. Create the file in **every** locale directory.
2. Add it to the `RawDictionary` type:

   ```ts
   export type RawDictionary = {
     common: typeof import("~/locales/en-US/common.json");
     auth: typeof import("~/locales/en-US/auth.json");
     // ... existing
     import: typeof import("~/locales/en-US/import.json");   // ← new
   };
   ```

3. Add the dynamic import in `fetchDictionary`:

   ```ts
   const [...existing, imp] = await Promise.all([
     // ... existing imports
     import(`~/locales/${locale}/import.json`).then((m) => m.default ?? m),
   ]);

   const raw: RawDictionary = { ...existing, import: imp };
   ```

### Step 5 — Test

```bash
cd web && npm run typecheck
```

TypeScript will catch any missing keys because the `RawDictionary` type
is derived from the `en-US` JSON structure.

---

## Backend Locale Files

Backend locales use [Project Fluent](https://projectfluent.org/) syntax
and live in `locales/{locale}/*.ftl`:

```
locales/
├── _meta.toml            # Locale metadata (name, native_name, etc.)
└── en-US/
    ├── auth.ftl           # Auth error messages
    ├── common.ftl         # Generic server messages
    └── errors.ftl         # Validation & domain error messages
```

### Fluent Syntax Basics

```ftl
# Simple message
welcome = Welcome to RustVault!

# Message with a variable
greeting = Welcome back, { $name }!

# Pluralisation
account-count = { $count ->
    [one] {$count} account
   *[other] {$count} accounts
}
```

### Adding a Backend Locale

1. Create `locales/de-DE/` with the same `.ftl` filenames.
2. Translate each message identifier (keep the same IDs).
3. Update `locales/_meta.toml` with the new locale's display names.
4. The Rust `I18nManager` auto-discovers locales at startup.

---

## Best Practices

1. **Always use the `t()` function** — never hard-code user-visible
   strings.
2. **Run `npm run typecheck`** — the type system ensures all keys exist.
3. **Keep parity** — every key in `en-US` must also exist in every
   other locale directory.
4. **Use descriptive key paths** — `banks.form.nameLabel` is better
   than `banks.label1`.
5. **Avoid string concatenation** — use template parameters instead of
   `t("hello") + " " + name`.
