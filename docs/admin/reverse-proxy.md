# Reverse Proxy Setup

This guide covers setting up a reverse proxy for RustChat.

## Recommended Proxies

- **Nginx** - Most common, battle-tested
- **Traefik** - Modern, Docker-native
- **Caddy** - Automatic HTTPS, simple config

## Why Use a Reverse Proxy?

- **TLS termination** - Handle HTTPS at the edge
- **Load balancing** - Distribute across multiple backends
- **WebSocket support** - Proper upgrade handling
- **Static file serving** - Cache frontend assets

## Nginx Configuration

### Basic Setup

```nginx
upstream rustchat_backend {
    server 127.0.0.1:3000;
}

server {
    listen 80;
    server_name chat.example.com;
    
    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name chat.example.com;
    
    # SSL certificates
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    
    # Proxy to backend
    location / {
        proxy_pass http://rustchat_backend;
        proxy_http_version 1.1;
        
        # WebSocket support
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Standard proxy headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
}
```

### Rate Limiting (Optional)

```nginx
# Rate limiting zones
limit_req_zone $binary_remote_addr zone=auth:10m rate=10r/m;
limit_req_zone $binary_remote_addr zone=api:10m rate=100r/m;

server {
    # ...
    
    # Apply stricter limits to auth endpoints
    location /api/v4/users/login {
        limit_req zone=auth burst=5 nodelay;
        proxy_pass http://rustchat_backend;
        # ...
    }
    
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        proxy_pass http://rustchat_backend;
        # ...
    }
}
```

## Traefik Configuration

### Docker Compose with Traefik

```yaml
version: '3.8'

services:
  traefik:
    image: traefik:v3.0
    command:
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--certificatesresolvers.letsencrypt.acme.tlschallenge=true"
      - "--certificatesresolvers.letsencrypt.acme.email=admin@example.com"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./letsencrypt:/letsencrypt

  backend:
    image: rustchat/backend
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.backend.rule=Host(`chat.example.com`)"
      - "traefik.http.routers.backend.tls=true"
      - "traefik.http.routers.backend.tls.certresolver=letsencrypt"
      - "traefik.http.services.backend.loadbalancer.server.port=3000"
      # WebSocket headers
      - "traefik.http.middlewares.backend-ws.headers.customRequestHeaders.Connection=upgrade"
      - "traefik.http.middlewares.backend-ws.headers.customRequestHeaders.Upgrade=websocket"
```

## Caddy Configuration

### Caddyfile

```
chat.example.com {
    reverse_proxy localhost:3000 {
        # WebSocket support
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
        
        # Forward headers
        header_up Host {host}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-For {remote}
        header_up X-Forwarded-Proto {scheme}
    }
    
    # Automatic HTTPS
    tls admin@example.com
}
```

## WebSocket Considerations

WebSocket connections require special handling:

1. **Upgrade headers** must be forwarded:
   ```
   Connection: upgrade
   Upgrade: websocket
   ```

2. **Timeouts** should be longer for WebSocket connections

3. **Load balancing** should use sticky sessions or shared state (Redis)

## Health Checks

Configure health checks in your proxy:

```nginx
# Nginx upstream health check
upstream rustchat_backend {
    server 127.0.0.1:3000 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3001 max_fails=3 fail_timeout=30s backup;
}
```

## Testing Your Setup

```bash
# Test HTTP
curl -I https://chat.example.com/api/v4/system/ping

# Test WebSocket
wss_url="wss://chat.example.com/api/v4/websocket"
# Use a WebSocket client to test connection

# Test with Mattermost mobile app
# Configure server URL: https://chat.example.com
```

---

*For security hardening: See [Security Guide](./security.md)*
