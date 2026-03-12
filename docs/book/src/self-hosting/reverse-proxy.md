# Reverse Proxy

In production, RustVault should sit behind a reverse proxy that handles TLS termination, compression, and optionally rate limiting.

## Caddy (Recommended)

Caddy automatically provisions Let's Encrypt certificates.

```
# Caddyfile
finance.example.com {
    reverse_proxy app:8080
}
```

Add Caddy to your `docker-compose.yml`:

```yaml
services:
  caddy:
    image: caddy:2-alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
    depends_on:
      - app

volumes:
  caddy_data:
```

## nginx

```nginx
server {
    listen 443 ssl http2;
    server_name finance.example.com;

    ssl_certificate     /etc/letsencrypt/live/finance.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/finance.example.com/privkey.pem;

    client_max_body_size 50M;

    location / {
        proxy_pass http://app:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name finance.example.com;
    return 301 https://$host$request_uri;
}
```

## Traefik

If you're using Traefik, add labels to the `app` service:

```yaml
services:
  app:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.rustvault.rule=Host(`finance.example.com`)"
      - "traefik.http.routers.rustvault.tls.certresolver=letsencrypt"
      - "traefik.http.services.rustvault.loadbalancer.server.port=8080"
```

## Important Notes

- **Upload size** — set `client_max_body_size` (nginx) or equivalent to at least the value of `server.max_upload_size` (default 50 MB), otherwise large file imports will fail at the proxy level.
- **WebSocket** — if future features add WebSocket support, ensure your proxy forwards `Upgrade` headers.
- **CORS** — when the frontend is served from a different domain, configure `server.allowed_origins` in `config.toml`.
