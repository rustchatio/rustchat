#!/bin/bash
#
# Deployment script for RustChat to Kubernetes
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"
NAMESPACE="rustchat"
ENVIRONMENT="${1:-staging}"

echo "🚀 Deploying RustChat to $ENVIRONMENT environment"

# Validate environment
if [[ "$ENVIRONMENT" != "staging" && "$ENVIRONMENT" != "production" ]]; then
    echo "❌ Error: Environment must be 'staging' or 'production'"
    exit 1
fi

# Check prerequisites
echo "🔍 Checking prerequisites..."

if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed"
    exit 1
fi

if ! command -v docker &> /dev/null; then
    echo "❌ docker is not installed"
    exit 1
fi

# Verify cluster connection
echo "🔌 Verifying cluster connection..."
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ Cannot connect to Kubernetes cluster"
    exit 1
fi

# Build and push images
echo "🏗️ Building Docker images..."
docker build -t rustchat-backend:latest -f docker/backend.Dockerfile backend/
docker build -t rustchat-frontend:latest -f docker/frontend.Dockerfile frontend-solid/

# Tag images for registry (adjust to your registry)
if [[ "$ENVIRONMENT" == "production" ]]; then
    REGISTRY="your-registry.com/rustchat"
    docker tag rustchat-backend:latest "$REGISTRY/backend:latest"
    docker tag rustchat-frontend:latest "$REGISTRY/frontend:latest"
    docker push "$REGISTRY/backend:latest"
    docker push "$REGISTRY/frontend:latest"
fi

# Create namespace if it doesn't exist
echo "📦 Creating namespace..."
kubectl apply -f "$INFRA_DIR/kubernetes/namespace.yaml"

# Apply secrets first
echo "🔐 Applying secrets..."
kubectl apply -f "$INFRA_DIR/kubernetes/secret.yaml"

# Apply ConfigMaps
echo "⚙️ Applying ConfigMaps..."
kubectl apply -f "$INFRA_DIR/kubernetes/configmap.yaml"

# Deploy dependencies
echo "🗄️ Deploying PostgreSQL..."
kubectl apply -f "$INFRA_DIR/kubernetes/postgres.yaml"

echo "📊 Deploying Redis..."
kubectl apply -f "$INFRA_DIR/kubernetes/redis.yaml"

# Wait for dependencies to be ready
echo "⏳ Waiting for dependencies to be ready..."
kubectl wait --for=condition=ready pod -l app=postgres -n "$NAMESPACE" --timeout=120s || true
kubectl wait --for=condition=ready pod -l app=redis -n "$NAMESPACE" --timeout=120s || true

# Deploy application
echo "🎯 Deploying backend..."
kubectl apply -f "$INFRA_DIR/kubernetes/backend.yaml"

echo "🎨 Deploying frontend..."
kubectl apply -f "$INFRA_DIR/kubernetes/frontend.yaml"

# Deploy ingress
echo "🌐 Deploying ingress..."
kubectl apply -f "$INFRA_DIR/kubernetes/ingress.yaml"

# Wait for rollout
echo "⏳ Waiting for rollout to complete..."
kubectl rollout status deployment/rustchat-backend -n "$NAMESPACE" --timeout=300s
kubectl rollout status deployment/rustchat-frontend -n "$NAMESPACE" --timeout=300s

# Verify deployment
echo "✅ Verifying deployment..."
kubectl get pods -n "$NAMESPACE"
kubectl get svc -n "$NAMESPACE"
kubectl get ingress -n "$NAMESPACE"

echo ""
echo "🎉 Deployment complete!"
echo ""
echo "Useful commands:"
echo "  kubectl get pods -n $NAMESPACE"
echo "  kubectl logs -f deployment/rustchat-backend -n $NAMESPACE"
echo "  kubectl logs -f deployment/rustchat-frontend -n $NAMESPACE"
