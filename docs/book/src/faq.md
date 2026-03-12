# FAQ

## General

### What is RustVault?

RustVault is a self-hosted personal finance application. You import bank statements, and RustVault helps you categorize, track, and analyse your transactions. It runs on your own server — your financial data never leaves your infrastructure.

### Is RustVault free?

Yes. RustVault is open-source software.

### What tech stack does RustVault use?

- **Backend:** Rust with Axum
- **Frontend:** SolidJS + TypeScript
- **Database:** PostgreSQL
- **Mobile:** Capacitor (planned)

## Deployment

### What are the minimum system requirements?

RustVault runs comfortably on 1 vCPU / 512 MB RAM. PostgreSQL needs additional resources depending on data size — 1 GB RAM total is a safe starting point.

### Can I run RustVault without Docker?

Yes. See [Installation — Building from Source](getting-started/installation.md#building-from-source) for instructions.

### Does RustVault need internet access?

No. RustVault runs entirely offline. Optional features like AI categorization or exchange rate fetching require internet, but the core application works without connectivity.

## Data & Import

### What bank statement formats are supported?

CSV, MT940, OFX/QFX, QIF, CAMT.053, XLSX/XLS/ODS, and JSON. See the [Import Formats](import-formats/csv.md) section for details on each.

### How does RustVault detect the file format?

Auto-detection uses three strategies: magic bytes (ZIP/OLE2), content inspection (format-specific markers), and file extension fallback.

### Can I import from multiple banks?

Yes. Create a bank and account for each institution, then import files into the corresponding account.

### How are duplicates handled?

RustVault checks for existing transactions with matching date, amount, and description in the same account. Potential duplicates are flagged during preview so you can skip them.

## Security

### How is authentication handled?

RustVault uses JWT access tokens (15-minute TTL) with HTTP-only refresh tokens (7-day TTL). Passwords are hashed with Argon2id. OIDC/SSO is also supported.

### Is my data encrypted?

Data is encrypted in transit (TLS via your reverse proxy). Optionally, sensitive fields can be encrypted at rest using AES-GCM when an `ENCRYPTION_KEY` is configured.

### Can I disable user registration after creating my account?

Yes. Set `auth.allow_new_user_register = false` in `config.toml` or restart with the updated configuration.

## Troubleshooting

### The app won't start — "database connection refused"

Ensure PostgreSQL is running and the `DATABASE_*` environment variables are correct. If using Docker Compose, verify the `db` service is healthy: `docker compose ps`.

### Import fails with "unsupported format"

Check that the file extension is in the `import.allowed_extensions` list in `config.toml`. If the format is supported but detection fails, try renaming the file with the correct extension.

### I forgot my password

If OIDC is not configured, there is currently no self-service password reset. Connect to the database and update the password hash directly, or delete and re-create the user.
