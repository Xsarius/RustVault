# Docker Setup

RustVault is distributed as a Docker image with a multi-stage build: Rust backend, Node.js frontend, and a minimal Debian runtime.

## Quick Start

```bash
mkdir rustvault && cd rustvault
curl -O https://raw.githubusercontent.com/xsarius/RustVault/main/docker/docker-compose.yml

# Create an .env file with your secrets
cat > .env << 'EOF'
JWT_SECRET=change-me-to-a-random-string-at-least-32-chars
DATABASE_PASSWORD=change-me-strong-password
EOF

docker compose up -d
```

The app is available at [http://localhost:8080](http://localhost:8080).

## Docker Compose File

The default `docker-compose.yml` starts two services:

| Service | Image | Purpose |
|---------|-------|---------|
| `app` | `ghcr.io/xsarius/rustvault:latest` | RustVault server + frontend |
| `db` | `postgres:17-alpine` | PostgreSQL database |

The database data is persisted in a named volume (`pgdata`).

## Image Details

The production image uses a 5-stage Dockerfile:

1. **Chef** — installs `cargo-chef` for Rust dependency caching
2. **Planner** — computes the dependency recipe
3. **Rust builder** — compiles dependencies (cached), then the application
4. **Node builder** — builds frontend assets with `npm run build`
5. **Runtime** — `debian:bookworm-slim` with only the compiled binary, static assets, and `config.toml`

The runtime image runs as a non-root `rustvault` user and exposes port 8080.

## Configuration

### Environment Variables

Secrets must be set via environment variables — see [Environment Variables](environment-variables.md) for the full reference.

The most important ones:

| Variable | Required | Description |
|----------|----------|-------------|
| `JWT_SECRET` | Yes | JWT signing key (≥ 32 characters) |
| `DATABASE_PASSWORD` | Yes | PostgreSQL password |
| `ENCRYPTION_KEY` | No | 64-hex AES-GCM key for field-level encryption |

### Config File

Non-secret settings are controlled via `config.toml`, which is baked into the image at `/app/config.toml`. To override, mount your own:

```yaml
volumes:
  - ./config.toml:/app/config.toml:ro
```

## Building the Image Locally

```bash
docker build -f docker/Dockerfile -t rustvault:local .
```

## Health Check

```bash
curl http://localhost:8080/api/health
# {"data":{"status":"healthy"}}
```

## What's Next?

- [Reverse Proxy](reverse-proxy.md) — put RustVault behind nginx or Caddy
- [Environment Variables](environment-variables.md) — full configuration reference
- [Backup & Recovery](backup-restore.md) — protect your data
