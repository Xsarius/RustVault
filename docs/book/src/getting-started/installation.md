# Installation

RustVault is a self-hosted application. The recommended deployment method is **Docker Compose**, which bundles the application server and a PostgreSQL database.

## Requirements

- [Docker](https://docs.docker.com/get-docker/) 24+ and Docker Compose v2
- PostgreSQL 16+ (included in Docker Compose)

## Quick Start (Docker Compose)

1. **Create a project directory**

   ```bash
   mkdir rustvault && cd rustvault
   ```

2. **Download the Compose file**

   ```bash
   curl -O https://raw.githubusercontent.com/xsarius/RustVault/main/docker/docker-compose.yml
   ```

3. **Create a `.env` file** with your secrets:

   ```bash
   cat > .env << 'EOF'
   # REQUIRED — at least 32 characters, random
   JWT_SECRET=change-me-to-a-random-string-at-least-32-chars

   # Database credentials
   DATABASE_USER=rustvault
   DATABASE_PASSWORD=change-me-strong-password
   DATABASE_NAME=rustvault

   # Optional — 64-hex-character key for field-level encryption
   # ENCRYPTION_KEY=

   # Optional — log verbosity (default: info)
   # RUST_LOG=info
   EOF
   ```

   > **Security:** Always generate strong random values for `JWT_SECRET` and `DATABASE_PASSWORD` in production. You can use `openssl rand -base64 48` to generate a suitable secret.

4. **Start the stack**

   ```bash
   docker compose up -d
   ```

5. **Verify it's running**

   ```bash
   curl http://localhost:8080/api/health
   # {"data":{"status":"healthy"}}
   ```

6. **Open the web UI** at [http://localhost:8080](http://localhost:8080) and register your first account.

## Building from Source

If you prefer not to use Docker:

### Prerequisites

- Rust 1.85+ (install via [rustup](https://rustup.rs))
- Node.js 22+ (or [Bun](https://bun.sh))
- PostgreSQL 16+
- [just](https://github.com/casey/just) command runner

### Steps

```bash
# Clone the repository
git clone https://github.com/xsarius/RustVault.git
cd RustVault

# Copy environment file and edit secrets
cp .env.example .env
$EDITOR .env

# Start only the database via Docker
just docker-db

# Build and run the backend
cargo run --release -p rustvault-server

# In another terminal, build and run the frontend dev server
cd web && npm install && npm run dev
```

The API will be available at `http://localhost:8080` and the dev frontend at `http://localhost:5173`.

## What's Next?

- [Setting Up Accounts](setting-up-accounts.md) — create banks and accounts
- [Environment Variables](../self-hosting/environment-variables.md) — full configuration reference
- [OIDC / SSO Setup](../self-hosting/oidc-setup.md) — enable single sign-on
