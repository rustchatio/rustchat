# RustChat Deployment Guide

This guide covers deploying RustChat to production using Docker Compose or Kubernetes.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Docker Compose Deployment](#docker-compose-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Configuration](#configuration)
- [Monitoring](#monitoring)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required

- Docker 24.0+ and Docker Compose v2.0+
- For Kubernetes: kubectl and a running cluster (1.27+)
- Domain name with DNS configured
- SSL certificates (Let's Encrypt recommended)

### Resources

| Component | CPU | Memory | Storage |
|-----------|-----|--------|---------|
| Backend (per replica) | 0.5 | 512MB | - |
| Frontend (per replica) | 0.1 | 128MB | - |
| PostgreSQL | 1.0 | 1GB | 50GB+ |
| Redis | 0.25 | 512MB | 10GB |
| MinIO/S3 | 0.5 | 512MB | 100GB+ |

## Docker Compose Deployment

### Quick Start

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/rustchat.git
   cd rustchat
   ```

2. **Configure environment:**
   ```bash
   cp .env.example .env
   # Edit .env with your settings
   ```

3. **Start services:**
   ```bash
   docker compose up -d --build
   ```

4. **Verify:**
   ```bash
   docker compose ps
   docker compose logs -f backend
   ```

### Production Configuration

Edit `.env` with production values:

```bash
# Security (REQUIRED)
RUSTCHAT_JWT_SECRET=your-secure-random-string-min-32-chars
RUSTCHAT_JWT_ISSUER=rustchat
RUSTCHAT_JWT_AUDIENCE=rustchat-users
RUSTCHAT_ENCRYPTION_KEY=your-32-byte-encryption-key

# URLs (REQUIRED)
RUSTCHAT_SITE_URL=https://chat.yourdomain.com
RUSTCHAT_CORS_ALLOWED_ORIGINS=https://chat.yourdomain.com

# Admin User (optional)
RUSTCHAT_ADMIN_USER=admin@yourdomain.com
RUSTCHAT_ADMIN_PASSWORD=secure-admin-password

# WebRTC Calls (optional)
RUSTCHAT_CALLS_ENABLED=true
RUSTCHAT_CALLS_UDP_PORT=8443
RUSTCHAT_CALLS_TCP_PORT=8443
```

### SSL with Traefik

For automatic SSL certificates:

```yaml
# docker-compose.prod.yml
services:
  traefik:
    image: traefik:v3.0
    command:
      - --providers.docker
      - --entrypoints.web.address=:80
      - --entrypoints.websecure.address=:443
      - --certificatesresolvers.letsencrypt.acme.tlschallenge=true
      - --certificatesresolvers.letsencrypt.acme.email=admin@yourdomain.com
      - --certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./letsencrypt:/letsencrypt
```

## Kubernetes Deployment

### Architecture

```
┌─────────────────┐
│   Ingress/Nginx  │
│   (SSL/TLS)      │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼───┐  ┌──▼────┐
│Frontend│  │Backend│
│  (2)   │  │  (3)   │
└────────┘  └───┬───┘
                │
        ┌───────┼───────┐
        │       │       │
    ┌───▼───┐ ┌─▼───┐ ┌─▼────┐
    │Postgres│ │Redis│ │MinIO │
    │  (1)   │ │ (1) │ │ (1)  │
    └────────┘ └─────┘ └──────┘
```

### Deployment Steps

1. **Create namespace:**
   ```bash
   kubectl apply -f infrastructure/kubernetes/namespace.yaml
   ```

2. **Configure secrets:**
   ```bash
   # Edit the secret file with your values
   vim infrastructure/kubernetes/secret.yaml
   kubectl apply -f infrastructure/kubernetes/secret.yaml
   ```

3. **Deploy dependencies:**
   ```bash
   kubectl apply -f infrastructure/kubernetes/postgres.yaml
   kubectl apply -f infrastructure/kubernetes/redis.yaml
   ```

4. **Deploy application:**
   ```bash
   kubectl apply -f infrastructure/kubernetes/backend.yaml
   kubectl apply -f infrastructure/kubernetes/frontend.yaml
   ```

5. **Configure ingress:**
   ```bash
   # Edit ingress.yaml with your domain
   vim infrastructure/kubernetes/ingress.yaml
   kubectl apply -f infrastructure/kubernetes/ingress.yaml
   ```

### Using the Deployment Script

```bash
# Deploy to staging
./infrastructure/scripts/deploy.sh staging

# Deploy to production
./infrastructure/scripts/deploy.sh production
```

### Rollback

```bash
# Rollback backend to previous version
./infrastructure/scripts/rollback.sh rustchat-backend

# Rollback to specific revision
./infrastructure/scripts/rollback.sh rustchat-backend 3
```

## Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `RUSTCHAT_JWT_SECRET` | JWT signing secret | - | Yes |
| `RUSTCHAT_JWT_ISSUER` | JWT issuer claim | rustchat | Yes |
| `RUSTCHAT_JWT_AUDIENCE` | JWT audience claim | rustchat-users | Yes |
| `RUSTCHAT_ENCRYPTION_KEY` | Data encryption key | - | Yes |
| `RUSTCHAT_DATABASE_URL` | PostgreSQL connection | - | Yes |
| `RUSTCHAT_REDIS_URL` | Redis connection | - | Yes |
| `RUSTCHAT_SITE_URL` | Public site URL | - | Yes |
| `RUSTCHAT_S3_ENDPOINT` | S3/MinIO endpoint | - | Yes |
| `RUSTCHAT_S3_ACCESS_KEY` | S3 access key | - | Yes |
| `RUSTCHAT_S3_SECRET_KEY` | S3 secret key | - | Yes |

### WebRTC Configuration

For calls to work properly:

1. **UDP port 8443 must be open** for media transport
2. **Configure ICE host override** if behind NAT:
   ```bash
   RUSTCHAT_CALLS_ICE_HOST_OVERRIDE=your-public-ip
   ```
3. **Optional: Configure TURN server** for NAT traversal:
   ```bash
   TURN_SERVER_ENABLED=true
   TURN_SERVER_URL=turn:turn.yourdomain.com:3478
   TURN_SERVER_USERNAME=turnuser
   TURN_SERVER_CREDENTIAL=turnpass
   ```

## Monitoring

### Setup

```bash
# Install Prometheus and Grafana
./infrastructure/scripts/monitoring-setup.sh
```

### Access Dashboards

```bash
# Port-forward Grafana
kubectl port-forward svc/grafana 3000:80 -n rustchat

# Access at http://localhost:3000
# Default credentials: admin/admin
```

### Available Metrics

- **HTTP Requests**: Rate, latency, errors
- **WebSocket Connections**: Active connections, messages/second
- **Database**: Connection pool, query performance
- **Calls**: Active calls, participants, SFU metrics
- **Infrastructure**: CPU, memory, disk usage

### Alerts

Configured alerts:
- High error rate (>10% for 5 min)
- High latency (p99 > 500ms for 5 min)
- Pod crash looping
- High memory usage (>85%)
- Database connection exhaustion

## Troubleshooting

### Common Issues

#### Backend fails to start

```bash
# Check logs
docker compose logs backend
kubectl logs -f deployment/rustchat-backend -n rustchat

# Common causes:
# - Database connection failed (check RUSTCHAT_DATABASE_URL)
# - Missing required secrets (check all env vars are set)
# - Port already in use
```

#### WebSocket connection fails

```bash
# Check WebSocket endpoint
curl -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" \
  -H "Sec-WebSocket-Key: test" \
  http://localhost:3000/api/v4/websocket

# Ensure:
# - CORS is configured correctly
# - JWT tokens are valid
# - WebSocket path is correct
```

#### Calls not working

```bash
# Test WebRTC connectivity
# 1. Check UDP port is open
nc -vu your-server 8443

# 2. Check ICE gathering
# In browser console during call:
# pc = new RTCPeerConnection();
# pc.createDataChannel('test');
# pc.createOffer().then(o => pc.setLocalDescription(o));
# pc.onicecandidate = e => console.log(e.candidate);
```

#### Database migrations failing

```bash
# Run migrations manually
docker compose exec backend cargo sqlx migrate run

# Or in Kubernetes:
kubectl exec -it deployment/rustchat-backend -- cargo sqlx migrate run
```

### Health Checks

```bash
# Backend health
curl http://localhost:3000/api/v1/health/live   # Liveness
curl http://localhost:3000/api/v1/health/ready  # Readiness

# Full stack test
./scripts/mm_compat_smoke.sh
./scripts/mm_mobile_smoke.sh
```

### Getting Help

- Check logs: `docker compose logs -f` or `kubectl logs -f`
- Review configuration: `cat .env` or `kubectl get configmap rustchat-config -o yaml`
- Test connectivity: `curl http://localhost:3000/api/v1/health/live`

## Security Checklist

- [ ] Changed all default passwords
- [ ] Generated strong JWT_SECRET (32+ chars)
- [ ] Generated unique ENCRYPTION_KEY
- [ ] Configured HTTPS with valid SSL certificate
- [ ] Set CORS_ALLOWED_ORIGINS to specific domains
- [ ] Disabled query token in production (`RUSTCHAT_SECURITY_WS_ALLOW_QUERY_TOKEN=false`)
- [ ] Enabled rate limiting
- [ ] Firewall configured (only 80, 443, 8443 exposed)
- [ ] Database not exposed publicly
- [ ] S3/MinIO credentials rotated
- [ ] Regular security updates scheduled
