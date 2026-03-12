# Backup & Recovery

RustVault stores all data in PostgreSQL. Regular backups are essential for any self-hosted deployment.

## Backup

### Docker Compose

Dump the database from the running `db` container:

```bash
docker compose exec db pg_dump -U rustvault rustvault > backup_$(date +%Y%m%d_%H%M%S).sql
```

### Compressed backup

```bash
docker compose exec db pg_dump -U rustvault -Fc rustvault > backup_$(date +%Y%m%d).dump
```

The `-Fc` flag produces a custom-format dump that is compressed and supports selective restore.

### Automated backups

Use cron (or systemd timers) to schedule nightly backups:

```bash
# /etc/cron.d/rustvault-backup
0 2 * * * root cd /opt/rustvault && docker compose exec -T db pg_dump -U rustvault -Fc rustvault > /backups/rustvault_$(date +\%Y\%m\%d).dump
```

Keep at least 7 daily and 4 weekly backups. Store copies off-site (S3, B2, rsync to another server).

## Restore

### From SQL dump

```bash
# Stop the app to avoid writes during restore
docker compose stop app

# Drop and recreate the database
docker compose exec db psql -U rustvault -c "DROP DATABASE rustvault;"
docker compose exec db psql -U rustvault -c "CREATE DATABASE rustvault;"

# Restore
docker compose exec -T db psql -U rustvault rustvault < backup.sql

# Restart the app
docker compose start app
```

### From custom-format dump

```bash
docker compose stop app
docker compose exec db psql -U rustvault -c "DROP DATABASE rustvault;"
docker compose exec db psql -U rustvault -c "CREATE DATABASE rustvault;"
docker compose exec -T db pg_restore -U rustvault -d rustvault < backup.dump
docker compose start app
```

## What to Back Up

| Data | Location | Method |
|------|----------|--------|
| Database | PostgreSQL `rustvault` database | `pg_dump` |
| Config | `config.toml`, `.env` | File copy |
| Docker volumes | `pgdata` named volume | `pg_dump` or volume backup |

Uploaded import files are **not** stored permanently — they are processed and discarded. Only the resulting transactions are saved in the database.
