# Environment Variables

RustVault uses a **layered configuration** approach:

1. **Environment variables** — secrets that must never be stored on disk
2. **`config.toml`** — operational tuning (pool sizes, timeouts, feature flags)
3. **Built-in defaults** — sensible values for everything else

> **Rule of thumb:** secrets go in env vars, everything else goes in `config.toml`.

## Secret Environment Variables

These variables are loaded **only** from the environment — never from the config file.

| Variable | Required | Description |
|----------|----------|-------------|
| `JWT_SECRET` | **Yes** | HMAC-SHA256 signing key for access tokens. Minimum 32 characters. Generate with `openssl rand -base64 48`. |
| `JWT_SECRET_OLD` | No | Previous JWT secret for zero-downtime key rotation. Set this to the old key when rotating `JWT_SECRET`. |
| `ENCRYPTION_KEY` | No | 64-character hex string (256-bit) for AES-GCM field-level encryption. Generate with `openssl rand -hex 32`. |
| `DATABASE_PASSWORD` | **Yes**\* | PostgreSQL password. \*Not required if `database.url` is set in `config.toml`. |
| `OIDC_CLIENT_ID` | No | OAuth 2.0 client ID from your OIDC provider. |
| `OIDC_CLIENT_SECRET` | No | OAuth 2.0 client secret from your OIDC provider. |
| `OIDC_ISSUER_URL` | No | OIDC discovery URL (e.g. `https://auth.example.com/application/o/rustvault/`). |

## Database Environment Variables

Used to construct the database connection string when `database.url` is empty in `config.toml`.

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_HOST` | `localhost` | PostgreSQL hostname |
| `DATABASE_PORT` | `5432` | PostgreSQL port |
| `DATABASE_USER` | `rustvault` | PostgreSQL username |
| `DATABASE_PASSWORD` | — | PostgreSQL password |
| `DATABASE_NAME` | `rustvault` | Database name |

The resulting URL is: `postgres://{USER}:{PASSWORD}@{HOST}:{PORT}/{NAME}`

## Operational Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUSTVAULT_CONFIG` | `config.toml` | Path to the TOML configuration file |
| `RUST_LOG` | `info` | Log verbosity filter (e.g. `debug`, `info,sqlx=warn`) |
| `PORT` | `8080` | Convenience alias used in `docker-compose.yml` port mapping |

## Config File Reference (`config.toml`)

All non-secret settings are controlled via the TOML configuration file. Below are all sections with their defaults.

### `[server]`

| Key | Default | Description |
|-----|---------|-------------|
| `port` | `8080` | HTTP listen port |
| `allowed_origins` | `[]` | CORS allowed origins (empty = permissive) |
| `request_timeout_secs` | `30` | Request timeout in seconds |
| `max_body_size` | `"10MB"` | Maximum JSON request body size |
| `max_upload_size` | `"50MB"` | Maximum file upload size |
| `locales_dir` | `"locales"` | Path to the i18n Fluent files directory |

### `[database]`

| Key | Default | Description |
|-----|---------|-------------|
| `url` | `""` | Full Postgres connection URL. If empty, built from `DATABASE_*` env vars. |
| `max_connections` | `10` | Maximum pool size |
| `min_connections` | `2` | Minimum idle connections |
| `acquire_timeout_secs` | `5` | Timeout when acquiring a connection from the pool |
| `idle_timeout_secs` | `300` | Idle connection timeout (5 minutes) |
| `max_lifetime_secs` | `1800` | Maximum connection lifetime (30 minutes) |

### `[auth]`

| Key | Default | Description |
|-----|---------|-------------|
| `allow_new_user_register` | `true` | Whether new users can self-register. Set to `false` after creating your account for single-user setups. |
| `access_token_ttl_secs` | `900` | Access token lifetime (15 minutes) |
| `refresh_token_ttl_secs` | `604800` | Refresh token lifetime (7 days) |
| `max_sessions_per_user` | `10` | Maximum concurrent sessions per user |
| `password_min_length` | `10` | Minimum password length |
| `password_max_length` | `128` | Maximum password length |
| `login_rate_limit_attempts` | `5` | Login attempts before rate limiting |
| `login_rate_limit_window_secs` | `900` | Rate limit window (15 minutes) |
| `account_lockout_threshold` | `20` | Failed attempts before account lockout |
| `audit_retention_days` | `90` | Days to retain audit log entries |

### `[auth.oidc]`

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `false` | Enable OIDC / SSO login |
| `display_name` | `"SSO"` | Button label shown on the login page |
| `scopes` | `["openid", "profile", "email"]` | OAuth scopes to request |
| `auto_register` | `true` | Automatically create accounts for new OIDC users |

### `[import]`

| Key | Default | Description |
|-----|---------|-------------|
| `max_file_size` | `"50MB"` | Maximum import file size |
| `allowed_extensions` | `["csv", "mt940", "sta", "ofx", ...]` | Allowed file extensions for import |

### `[ai]`

| Key | Default | Description |
|-----|---------|-------------|
| `enabled` | `false` | Enable AI-powered features (auto-categorisation, etc.) |

## Example `.env` File

```bash
# Required secrets
JWT_SECRET=your-random-secret-at-least-32-characters-long
DATABASE_PASSWORD=strong-database-password

# Database connection
DATABASE_HOST=db
DATABASE_PORT=5432
DATABASE_USER=rustvault
DATABASE_NAME=rustvault

# Optional — field-level encryption
# ENCRYPTION_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

# Optional — OIDC / SSO
# OIDC_ISSUER_URL=https://auth.example.com/application/o/rustvault/
# OIDC_CLIENT_ID=rustvault
# OIDC_CLIENT_SECRET=your-client-secret

# Logging
RUST_LOG=info
```
