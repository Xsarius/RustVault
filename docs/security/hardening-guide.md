# Production Hardening Guide

> Checklist and guidance for securely deploying RustVault in a production self-hosted environment.
> Follow these steps **before** exposing your instance to the internet.

---

## Table of Contents

1. [Quick Checklist](#1-quick-checklist)
2. [TLS / HTTPS Configuration](#2-tls--https-configuration)
3. [Reverse Proxy Setup](#3-reverse-proxy-setup)
4. [Firewall & Network](#4-firewall--network)
5. [Database Security](#5-database-security)
6. [Docker Hardening](#6-docker-hardening)
7. [Secret Management](#7-secret-management)
8. [Backup & Recovery](#8-backup--recovery)
9. [Monitoring & Logging](#9-monitoring--logging)
10. [Update Strategy](#10-update-strategy)
11. [Optional Hardening](#11-optional-hardening)

---

## 1. Quick Checklist

Use this as a pass/fail checklist before going to production. Each item links to the detailed section below.

### Critical (Must Do)

- [ ] [TLS termination configured](#2-tls--https-configuration) — all traffic over HTTPS
- [ ] [Strong `JWT_SECRET` generated](#71-jwt-signing-key) — minimum 256-bit random key
- [ ] [Database password set](#51-authentication) — not using default credentials
- [ ] [Database port not exposed](#52-network-isolation) — PostgreSQL only on internal Docker network
- [ ] [`.env` file permissions](#73-file-permissions) — readable only by root/deployer (mode `0600`)
- [ ] [Backups configured](#8-backup--recovery) — automated with encryption
- [ ] [Firewall rules applied](#4-firewall--network) — only ports 80/443 open to internet

### Recommended

- [ ] [Reverse proxy in front of RustVault](#3-reverse-proxy-setup) — Caddy, Nginx, or Traefik
- [ ] [HSTS enabled](#24-hsts) — with `includeSubDomains`
- [ ] [Fail2ban configured](#94-fail2ban-integration) — block repeated auth failures
- [ ] [Log rotation set up](#92-log-rotation) — prevent disk exhaustion
- [ ] [Automatic updates](#10-update-strategy) — Watchtower or scheduled pulls
- [ ] [CORS origins restricted](#76-cors-configuration) — only your domain(s)

### Optional (Defense in Depth)

- [ ] [Database TDE or volume encryption](#55-encryption-at-rest)
- [ ] [Separate database host](#53-separate-database-host)
- [ ] [Network segmentation with VLANs](#44-network-segmentation)
- [ ] [Container image scanning](#67-image-scanning)
- [ ] [VPN access only](#45-vpn-only-access) — no public internet exposure

---

## 2. TLS / HTTPS Configuration

### 2.1 Why TLS Is Required

RustVault transmits authentication tokens and financial data. **All production deployments must use TLS.** Without TLS:
- Access tokens can be intercepted (session hijacking)
- Financial data is readable by network observers
- `Secure` cookie flag prevents refresh tokens from being sent over HTTP
- HSTS cannot be enabled

### 2.2 Option A: Caddy (Recommended — Automatic TLS)

Caddy automatically obtains and renews Let's Encrypt certificates.

```Caddyfile
# /etc/caddy/Caddyfile
finance.example.com {
    reverse_proxy rustvault-server:8080

    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
        Referrer-Policy "strict-origin-when-cross-origin"
    }

    log {
        output file /var/log/caddy/access.log
        format json
    }
}
```

Add to `docker-compose.yml`:

```yaml
services:
  caddy:
    image: caddy:2-alpine
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    networks:
      - rustvault-net

  rustvault-server:
    # ... existing config ...
    # Remove port mapping — Caddy handles external traffic
    expose:
      - "8080"
    networks:
      - rustvault-net

volumes:
  caddy_data:
  caddy_config:
```

### 2.3 Option B: Nginx with Let's Encrypt (Certbot)

```nginx
# /etc/nginx/sites-available/rustvault
server {
    listen 443 ssl http2;
    server_name finance.example.com;

    ssl_certificate /etc/letsencrypt/live/finance.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/finance.example.com/privkey.pem;

    # Modern TLS configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:10m;
    ssl_session_tickets off;

    # OCSP stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    resolver 1.1.1.1 1.0.0.1 valid=300s;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # Rate limit login endpoint (additional layer)
    location /api/auth/login {
        limit_req zone=login burst=5 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name finance.example.com;
    return 301 https://$host$request_uri;
}
```

### 2.4 HSTS

When you are confident your TLS setup is working correctly, enable HSTS to prevent downgrade attacks:

```
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
```

> **Warning:** Only add `preload` after verifying your domain works over HTTPS. Preloading is permanent and hard to undo. Submit at [hstspreload.org](https://hstspreload.org/) only when ready.

---

## 3. Reverse Proxy Setup

### 3.1 Why Use a Reverse Proxy

| Benefit | Description |
|---------|-------------|
| **TLS termination** | Handles certificate management (Let's Encrypt auto-renewal) |
| **Request buffering** | Protects the app from slow clients |
| **Additional rate limiting** | Defense-in-depth — rate limit at proxy AND application level |
| **Access logging** | Separate access logs in standard format |
| **Static asset caching** | Cache SPA assets at the proxy layer |
| **IP filtering** | Block/allow by IP range before reaching the app |

### 3.2 Trusted Proxy Configuration

RustVault reads `X-Forwarded-For` for rate limiting. **You must configure the trusted proxy** so attackers cannot spoof their IP:

```bash
# In your .env or environment
TRUSTED_PROXIES=172.18.0.0/16  # Docker internal network CIDR
```

RustVault will only trust `X-Forwarded-For` from IPs in this range.

### 3.3 WebSocket Proxying

For real-time import progress and notifications, ensure your reverse proxy passes WebSocket connections:

| Proxy | Required Config |
|-------|----------------|
| **Caddy** | Automatic — no extra config needed |
| **Nginx** | `proxy_http_version 1.1;` + `Upgrade`/`Connection` headers (see Nginx example above) |
| **Traefik** | Automatic with Docker provider |

---

## 4. Firewall & Network

### 4.1 Minimal Open Ports

Only two ports should be exposed to the internet:

| Port | Protocol | Purpose |
|------|----------|---------|
| **80** | TCP | HTTP → HTTPS redirect only |
| **443** | TCP | HTTPS (TLS-terminated by reverse proxy) |

**All other ports must be firewalled**, including:
- Port **5432** (PostgreSQL) — **never expose to internet**
- Port **8080** (RustVault internal) — only accessible from reverse proxy
- Port **11434** (Ollama, if used) — internal access only

### 4.2 UFW Example (Ubuntu/Debian)

```bash
# Reset to deny all
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (restrict to your IP if possible)
sudo ufw allow from YOUR_IP to any port 22

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Enable
sudo ufw enable
sudo ufw status verbose
```

### 4.3 iptables Example

```bash
# Allow established connections
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# Allow loopback
iptables -A INPUT -i lo -j ACCEPT

# Allow SSH (from specific IP)
iptables -A INPUT -p tcp --dport 22 -s YOUR_IP -j ACCEPT

# Allow HTTP/HTTPS
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Drop everything else
iptables -A INPUT -j DROP
```

### 4.4 Network Segmentation

For advanced setups, place the database on a separate VLAN or Docker network with no internet access:

```yaml
# docker-compose.yml
networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true  # No internet access

services:
  caddy:
    networks:
      - frontend

  rustvault-server:
    networks:
      - frontend
      - backend

  postgres:
    networks:
      - backend  # Only reachable from rustvault-server
```

### 4.5 VPN-Only Access

For maximum security, don't expose RustVault to the internet at all. Access through a VPN (WireGuard, Tailscale, ZeroTier):

```bash
# Allow access only from VPN interface
sudo ufw allow in on wg0 to any port 443
```

This eliminates almost all external attack surface.

---

## 5. Database Security

### 5.1 Authentication

**Never use default credentials in production.**

```bash
# Generate a strong random password
openssl rand -base64 32

# Set in .env — use DATABASE_* env vars (mapped to PG* internally)
DATABASE_HOST=postgres
DATABASE_PORT=5432
DATABASE_USER=rustvault
DATABASE_PASSWORD=YOUR_GENERATED_PASSWORD
DATABASE_NAME=rustvault
```

RustVault maps these to the [standard libpq environment variables](https://www.postgresql.org/docs/current/libpq-envars.html) (`PGHOST`, `PGUSER`, etc.) internally. Both the RustVault server and the PostgreSQL container use the same password value:

```yaml
# docker-compose.yml
services:
  postgres:
    environment:
      POSTGRES_USER: ${DATABASE_USER}
      POSTGRES_PASSWORD: ${DATABASE_PASSWORD}
      POSTGRES_DB: ${DATABASE_NAME}

  rustvault-server:
    environment:
      DATABASE_HOST: postgres
      DATABASE_PORT: "5432"
      DATABASE_USER: ${DATABASE_USER}
      DATABASE_PASSWORD: ${DATABASE_PASSWORD}
      DATABASE_NAME: ${DATABASE_NAME}
```

### 5.2 Network Isolation

PostgreSQL should **only** be accessible from the RustVault application container:

```yaml
# docker-compose.yml
services:
  postgres:
    # NO ports: section — do not expose to host
    networks:
      - internal

  rustvault-server:
    networks:
      - internal

networks:
  internal:
    driver: bridge
```

Verify PostgreSQL is not exposed:

```bash
# Should return nothing / connection refused
nmap -p 5432 your-server-ip
```

### 5.3 Separate Database Host

For high-value deployments, run PostgreSQL on a dedicated host:

```bash
# PostgreSQL pg_hba.conf — allow only the app server IP
host    rustvault    rustvault    10.0.1.5/32    scram-sha-256
```

```bash
# .env on the app server
DATABASE_HOST=10.0.1.10
DATABASE_PORT=5432
DATABASE_USER=rustvault
DATABASE_PASSWORD=PASSWORD
DATABASE_NAME=rustvault
```

### 5.4 PostgreSQL Configuration Hardening

Add to `postgresql.conf`:

```ini
# Disable remote superuser login
listen_addresses = '0.0.0.0'       # Or specific interface

# Connection limits
max_connections = 20                # Match app pool size + buffer
superuser_reserved_connections = 3

# Authentication
password_encryption = scram-sha-256

# Logging (detect anomalies)
log_connections = on
log_disconnections = on
log_statement = 'ddl'              # Log schema changes
log_min_duration_statement = 1000  # Log slow queries (>1s)

# SSL (if DB is on separate host)
ssl = on
ssl_cert_file = '/etc/ssl/certs/server.crt'
ssl_key_file = '/etc/ssl/private/server.key'
```

### 5.5 Encryption at Rest

#### Option A: Volume Encryption (Recommended)

Encrypt the Docker volume or host directory where PostgreSQL data resides:

```bash
# LUKS encryption on Linux
cryptsetup luksFormat /dev/sdb1
cryptsetup open /dev/sdb1 pgdata
mkfs.ext4 /dev/mapper/pgdata
mount /dev/mapper/pgdata /var/lib/rustvault/pgdata
```

#### Option B: PostgreSQL TDE

PostgreSQL 18+ supports Transparent Data Encryption via extensions. Consult the PostgreSQL documentation for your version.

---

## 6. Docker Hardening

### 6.1 Non-Root Execution

The official RustVault image runs as a non-root user by default. Verify:

```bash
docker exec rustvault-server whoami
# Expected: rustvault
```

### 6.2 Read-Only Filesystem

```yaml
# docker-compose.yml
services:
  rustvault-server:
    read_only: true
    tmpfs:
      - /tmp:size=100M
    volumes:
      - import_temp:/tmp/rustvault-imports
```

### 6.3 Drop All Capabilities

```yaml
services:
  rustvault-server:
    cap_drop:
      - ALL
    security_opt:
      - no-new-privileges:true
```

### 6.4 Resource Limits

Prevent resource exhaustion attacks:

```yaml
services:
  rustvault-server:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 128M

  postgres:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M
```

### 6.5 Health Checks

```yaml
services:
  rustvault-server:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

  postgres:
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rustvault"]
      interval: 10s
      timeout: 5s
      retries: 5
```

### 6.6 Pin Image Versions

Always pin to specific digests or version tags, never use `latest`:

```yaml
services:
  rustvault-server:
    image: ghcr.io/xsarius/rustvault:1.2.3
    # Or use digest: ghcr.io/xsarius/rustvault@sha256:abc123...
```

### 6.7 Image Scanning

Scan images before deployment:

```bash
# With Trivy
trivy image ghcr.io/xsarius/rustvault:1.2.3

# With Grype
grype ghcr.io/xsarius/rustvault:1.2.3
```

Block deployment on critical or high CVEs.

---

## 7. Secret Management

### 7.1 JWT Signing Key

Generate a strong random key:

```bash
# Generate a 256-bit (32-byte) key
openssl rand -base64 32
# Example output: K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols=
```

Set in your environment:

```bash
JWT_SECRET=K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols=
```

**Key rotation:** When rotating, set the old key to allow existing tokens to validate during the transition:

```bash
JWT_SECRET=<new-key>
JWT_SECRET_OLD=<previous-key>
```

After all access tokens expire (15 minutes) and refresh tokens rotate (up to 7 days), remove `JWT_SECRET_OLD`.

### 7.2 Docker Secrets (Preferred for Compose)

```bash
# Create secret files
mkdir -p secrets
openssl rand -base64 32 > secrets/jwt_secret.txt
chmod 600 secrets/*
```

```yaml
# docker-compose.yml
services:
  rustvault-server:
    secrets:
      - jwt_secret
    environment:
      JWT_SECRET_FILE: /run/secrets/jwt_secret
      DATABASE_HOST: postgres
      DATABASE_PORT: "5432"
      DATABASE_USER: rustvault
      DATABASE_PASSWORD: ${DATABASE_PASSWORD}   # From .env file
      DATABASE_NAME: rustvault

secrets:
  jwt_secret:
    file: ./secrets/jwt_secret.txt
```

### 7.3 File Permissions

```bash
# .env file — readable only by owner
chmod 600 .env

# Secrets directory
chmod 700 secrets/
chmod 600 secrets/*

# Docker socket (if applicable)
# Do NOT mount Docker socket into containers
```

### 7.4 Environment Variable Checklist

| Variable | Required | Sensitive | Description |
|----------|----------|-----------|-------------|
| `DATABASE_HOST` | Yes | No | PostgreSQL hostname (e.g. `postgres`) |
| `DATABASE_PORT` | No | No | PostgreSQL port (default `5432`) |
| `DATABASE_USER` | Yes | No | PostgreSQL username |
| `DATABASE_PASSWORD` | Yes | **Yes** | PostgreSQL password |
| `DATABASE_NAME` | Yes | No | PostgreSQL database name |
| `JWT_SECRET` | Yes | **Critical** | JWT signing key (min 256-bit) |
| `JWT_SECRET_OLD` | No | **Critical** | Previous JWT key (rotation) |
| `ENCRYPTION_KEY` | No | **Critical** | AES key for AI config encryption |
| `OIDC_CLIENT_SECRET` | No | Yes | OIDC provider client secret |
| `ALLOWED_ORIGINS` | No | No | CORS origin allowlist |
| `BIND_ADDRESS` | No | No | Listen address (default `0.0.0.0:8080`) |
| `RUST_LOG` | No | No | Log level filter |

### 7.5 Never Commit Secrets

Ensure `.gitignore` includes:

```gitignore
.env
secrets/
*.key
*.pem
```

### 7.6 CORS Configuration

Restrict allowed origins to your actual domain:

```bash
ALLOWED_ORIGINS=https://finance.example.com
```

Multiple origins (comma-separated):

```bash
ALLOWED_ORIGINS=https://finance.example.com,https://finance-staging.example.com
```

**Never use `*` in production.**

---

## 8. Backup & Recovery

### 8.1 Automated Backups

#### Option A: RustVault Built-in Backup (Admin API)

```bash
# Trigger encrypted backup via API
curl -X POST https://finance.example.com/api/admin/backup \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"encryption_key": "your-backup-passphrase"}' \
  -o backup-$(date +%Y%m%d).enc
```

#### Option B: pg_dump Cron Job

```bash
#!/bin/bash
# /usr/local/bin/rustvault-backup.sh

BACKUP_DIR="/backups/rustvault"
RETENTION_DAYS=30
DATE=$(date +%Y%m%d_%H%M%S)

# Dump database
docker exec rustvault-postgres pg_dump \
  -U rustvault \
  -d rustvault \
  --format=custom \
  --compress=9 \
  > "$BACKUP_DIR/rustvault_$DATE.dump"

# Encrypt with GPG
gpg --symmetric \
  --cipher-algo AES256 \
  --batch --passphrase-file /root/.backup-passphrase \
  "$BACKUP_DIR/rustvault_$DATE.dump"

# Remove unencrypted dump
rm "$BACKUP_DIR/rustvault_$DATE.dump"

# Prune old backups
find "$BACKUP_DIR" -name "*.dump.gpg" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: rustvault_$DATE.dump.gpg"
```

```bash
# Crontab — daily at 03:00
0 3 * * * /usr/local/bin/rustvault-backup.sh >> /var/log/rustvault-backup.log 2>&1
```

### 8.2 Backup Storage

| Location | Pros | Cons |
|----------|------|------|
| **Local disk** | Fast, no dependencies | Single point of failure |
| **External drive / NAS** | Survives host failure | Still on-site |
| **Cloud storage (S3/B2/R2)** | Off-site, durable | Requires upload script, egress costs |
| **rsync to remote host** | Simple, off-site | Requires another server |

**Best practice:** At least two backup locations — **local + off-site**.

### 8.3 Backup Verification

Regularly test restoration:

```bash
# Restore to a test database
docker exec -i rustvault-postgres pg_restore \
  -U rustvault \
  -d rustvault_test \
  --clean --if-exists \
  < backup.dump
```

Schedule monthly restore tests. An untested backup is not a backup.

### 8.4 Backup Encryption

**All backups should be encrypted.** Financial data in a plaintext database dump is a high-value target.

- Built-in backups use AES-256 with a user-provided key
- pg_dump backups should be encrypted with GPG (symmetric) or `age`
- Store the encryption key/passphrase separately from the backup (e.g., password manager)

---

## 9. Monitoring & Logging

### 9.1 Structured Logs

RustVault outputs JSON-formatted logs by default. Key fields:

```json
{
  "timestamp": "2026-03-03T10:15:30Z",
  "level": "WARN",
  "target": "rustvault_server::middleware::auth",
  "message": "Authentication failed",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "ip": "192.168.1.100",
  "path": "/api/auth/login",
  "reason": "invalid_credentials"
}
```

### 9.2 Log Rotation

Prevent logs from filling the disk:

```yaml
# docker-compose.yml
services:
  rustvault-server:
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "5"
```

### 9.3 Security Events to Monitor

| Event | Log Level | Action |
|-------|-----------|--------|
| Failed login attempts | WARN | Alert on >10 per IP in 1 hour |
| Token reuse detected | ERROR | Investigate — potential token theft |
| Rate limit triggered | WARN | Monitor for sustained attacks |
| Admin actions (backup, user management) | INFO | Audit trail |
| Account lockout | WARN | Notify admin |
| OIDC authentication failure | WARN | Check provider configuration |
| Import of unusually large file | INFO | Review if unexpected |

### 9.4 Fail2ban Integration

RustVault logs failed authentication in a format compatible with fail2ban:

```ini
# /etc/fail2ban/filter.d/rustvault.conf
[Definition]
failregex = .*"ip":"<HOST>".*"message":"Authentication failed".*
            .*"ip":"<HOST>".*"message":"Rate limit exceeded".*
ignoreregex =
```

```ini
# /etc/fail2ban/jail.d/rustvault.conf
[rustvault]
enabled = true
port = 443
filter = rustvault
logpath = /var/lib/docker/containers/*rustvault-server*/*-json.log
maxretry = 10
findtime = 600
bantime = 3600
```

### 9.5 Uptime Monitoring

Monitor the health endpoint externally:

```bash
# Simple check
curl -sf https://finance.example.com/api/health || echo "RustVault is DOWN"
```

Use an external service (Uptime Kuma, Healthchecks.io, Uptime Robot) to alert on downtime.

---

## 10. Update Strategy

### 10.1 Manual Updates

```bash
# Pull latest image
docker compose pull

# Recreate containers (database is preserved in volume)
docker compose up -d

# Verify health
curl -sf https://finance.example.com/api/health
```

### 10.2 Automatic Updates (Watchtower)

```yaml
services:
  watchtower:
    image: containrrr/watchtower
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      WATCHTOWER_SCHEDULE: "0 0 4 * * *"  # Daily at 04:00
      WATCHTOWER_CLEANUP: "true"
      WATCHTOWER_INCLUDE_STOPPED: "false"
      WATCHTOWER_NOTIFICATIONS: "email"
    restart: unless-stopped
```

> **Caution:** Automatic updates may introduce breaking changes. For critical deployments, prefer manual updates with changelog review.

### 10.3 Database Migrations

RustVault applies database migrations automatically on startup. Before a major version upgrade:

1. Create a backup (see [§8](#8-backup--recovery))
2. Review the changelog for breaking changes
3. Update the image
4. Verify the health endpoint
5. Spot-check data in the UI

---

## 11. Optional Hardening

### 11.1 Content Security Policy Tightening

If you host RustVault on a dedicated subdomain with no third-party integrations, you can tighten the CSP at the reverse proxy level:

```
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self'; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'
```

### 11.2 DNS Configuration

```
# CAA record — restrict certificate issuance
finance.example.com. IN CAA 0 issue "letsencrypt.org"

# DNSSEC — if supported by your registrar
```

### 11.3 Audit Logging to External System

Forward structured logs to a SIEM or log aggregation service:

```yaml
services:
  rustvault-server:
    logging:
      driver: syslog
      options:
        syslog-address: "tcp://logserver:514"
        tag: "rustvault"
```

### 11.4 Two-Factor Authentication

If your OIDC provider supports MFA, enable it there. RustVault delegates 2FA to the OIDC provider:

1. Configure Authentik/Keycloak with TOTP or WebAuthn
2. Enable OIDC in RustVault
3. Users authenticate via the OIDC provider's MFA flow

### 11.5 IP Allowlisting

For private deployments, restrict access to known IPs at the reverse proxy:

```nginx
# Nginx — allow only specific IPs
location / {
    allow 10.0.0.0/8;
    allow 192.168.1.0/24;
    deny all;
    proxy_pass http://127.0.0.1:8080;
}
```

---

## Complete Production `docker-compose.yml`

```yaml
version: "3.8"

services:
  caddy:
    image: caddy:2-alpine
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    networks:
      - frontend
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"

  rustvault-server:
    image: ghcr.io/xsarius/rustvault:1.0.0  # Pin version
    restart: unless-stopped
    read_only: true
    tmpfs:
      - /tmp:size=100M
    cap_drop:
      - ALL
    security_opt:
      - no-new-privileges:true
    secrets:
      - jwt_secret
    environment:
      DATABASE_HOST: postgres
      DATABASE_PORT: "5432"
      DATABASE_USER: rustvault
      DATABASE_PASSWORD: ${DATABASE_PASSWORD}
      DATABASE_NAME: rustvault
      JWT_SECRET_FILE: /run/secrets/jwt_secret
      ALLOWED_ORIGINS: https://finance.example.com
      RUST_LOG: rustvault=info,tower_http=info
    expose:
      - "8080"
    networks:
      - frontend
      - backend
    depends_on:
      postgres:
        condition: service_healthy
    deploy:
      resources:
        limits:
          cpus: "2.0"
          memory: 512M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "5"

  postgres:
    image: postgres:18-alpine  # Pin major version
    restart: unless-stopped
    environment:
      POSTGRES_USER: rustvault
      POSTGRES_DB: rustvault
      POSTGRES_PASSWORD: ${DATABASE_PASSWORD}
    volumes:
      - pgdata:/var/lib/postgresql/data
    networks:
      - backend
    deploy:
      resources:
        limits:
          cpus: "2.0"
          memory: 1G
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rustvault"]
      interval: 10s
      timeout: 5s
      retries: 5
    logging:
      driver: json-file
      options:
        max-size: "5m"
        max-file: "3"

networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true

volumes:
  caddy_data:
  caddy_config:
  pgdata:

secrets:
  jwt_secret:
    file: ./secrets/jwt_secret.txt
```

---

## References

- [Docker Security Best Practices](https://docs.docker.com/engine/security/)
- [PostgreSQL Security](https://www.postgresql.org/docs/current/security.html)
- [Caddy Documentation](https://caddyserver.com/docs/)
- [Mozilla SSL Configuration Generator](https://ssl-config.mozilla.org/)
- [OWASP Docker Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html)
- [RustVault Threat Model](threat-model.md)
- [RustVault Security Policy](SECURITY.md)
