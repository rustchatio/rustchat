# RustChat Infrastructure

This directory contains all infrastructure configuration for deploying RustChat to production.

## Directory Structure

```
infrastructure/
├── kubernetes/          # Kubernetes manifests
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── secret.yaml
│   ├── postgres.yaml
│   ├── redis.yaml
│   ├── backend.yaml
│   ├── frontend.yaml
│   └── ingress.yaml
├── monitoring/          # Prometheus & Grafana configs
│   ├── prometheus.yaml
│   ├── grafana-dashboards.yaml
│   └── grafana-datasources.yaml
└── scripts/             # Deployment scripts
    ├── deploy.sh
    ├── rollback.sh
    └── monitoring-setup.sh
```

## Quick Start

### Docker Compose (Development)

```bash
# From project root
docker compose up -d --build
```

### Kubernetes (Production)

```bash
# Using deployment script
./infrastructure/scripts/deploy.sh production

# Or manually
kubectl apply -f infrastructure/kubernetes/
```

### Monitoring Setup

```bash
./infrastructure/scripts/monitoring-setup.sh
```

## Configuration

### Required Secrets

Edit `kubernetes/secret.yaml` with your values:

```yaml
stringData:
  RUSTCHAT_JWT_SECRET: "your-secure-secret"
  RUSTCHAT_ENCRYPTION_KEY: "your-encryption-key"
  RUSTCHAT_DATABASE_URL: "postgres://..."
  RUSTCHAT_S3_ACCESS_KEY: "..."
  RUSTCHAT_S3_SECRET_KEY: "..."
```

Apply:
```bash
kubectl apply -f infrastructure/kubernetes/secret.yaml
```

## Scaling

### Horizontal Pod Autoscaler

Backend and frontend deployments include HPAs:

```bash
# View current replicas
kubectl get hpa -n rustchat

# Scale manually (if needed)
kubectl scale deployment rustchat-backend --replicas=5 -n rustchat
```

## Monitoring

Access Grafana:
```bash
kubectl port-forward svc/grafana 3000:80 -n rustchat
# Open http://localhost:3000
```

Default dashboards:
- RustChat Overview
- RustChat Calls

## Troubleshooting

```bash
# Check pod status
kubectl get pods -n rustchat

# View logs
kubectl logs -f deployment/rustchat-backend -n rustchat

# Check events
kubectl get events -n rustchat --sort-by='.lastTimestamp'

# Exec into pod
kubectl exec -it deployment/rustchat-backend -n rustchat -- /bin/sh
```

## Rollback

```bash
# Rollback to previous version
./infrastructure/scripts/rollback.sh rustchat-backend

# Rollback to specific revision
./infrastructure/scripts/rollback.sh rustchat-backend 3
```

## Resources

- [Deployment Guide](../docs/DEPLOYMENT.md)
- [Rollout Strategy](../docs/ROLLOUT.md)
