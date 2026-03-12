# Upgrading

RustVault uses database migrations that run automatically on startup. Upgrading is typically a one-command operation.

## Docker Compose

```bash
# Pull the latest image
docker compose pull

# Restart with the new version
docker compose up -d
```

On startup, RustVault runs any pending database migrations automatically. No manual migration step is needed.

## Before Upgrading

1. **Read the release notes** — check for breaking changes or required configuration updates.
2. **Back up the database** — always create a [backup](backup-restore.md) before upgrading:
   ```bash
   docker compose exec db pg_dump -U rustvault -Fc rustvault > backup_pre_upgrade.dump
   ```
3. **Back up your `.env` and `config.toml`** — in case the upgrade introduces new configuration keys.

## Pinning a Version

To avoid unexpected upgrades, pin the image tag in your `docker-compose.yml`:

```yaml
services:
  app:
    image: ghcr.io/xsarius/rustvault:0.4.0
```

## Rolling Back

If an upgrade causes issues:

1. Stop the app: `docker compose stop app`
2. Restore the database from your backup (see [Backup & Recovery](backup-restore.md))
3. Change the image tag back to the previous version
4. Start the app: `docker compose up -d`

> **Note:** Rolling back database migrations is not automatic. Always restore from a pre-upgrade backup when downgrading.

## Building from Source

If you build from source, pull the latest code, rebuild, and restart:

```bash
git pull origin main
cargo build --release -p rustvault-server
# Restart your systemd service or process manager
```
