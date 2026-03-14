#!/bin/bash
#
# Setup monitoring stack for RustChat
#

set -e

NAMESPACE="rustchat"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"

echo "📊 Setting up monitoring stack..."

# Add Helm repositories
echo "📦 Adding Helm repositories..."
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update

# Install Prometheus
echo "🔥 Installing Prometheus..."
helm upgrade --install prometheus prometheus-community/prometheus \
    --namespace "$NAMESPACE" \
    --create-namespace \
    --set server.persistentVolume.enabled=true \
    --set server.persistentVolume.size=10Gi \
    --set alertmanager.persistentVolume.enabled=true \
    --set alertmanager.persistentVolume.size=5Gi

# Apply custom Prometheus config
echo "⚙️ Applying custom Prometheus configuration..."
kubectl apply -f "$INFRA_DIR/monitoring/prometheus.yaml"

# Install Grafana
echo "📈 Installing Grafana..."
helm upgrade --install grafana grafana/grafana \
    --namespace "$NAMESPACE" \
    --set persistence.enabled=true \
    --set persistence.size=5Gi \
    --set admin.password='admin' \
    --set datasources."datasources\\.yaml".apiVersion=1 \
    --set datasources."datasources\\.yaml".datasources[0].name=Prometheus \
    --set datasources."datasources\\.yaml".datasources[0].type=prometheus \
    --set datasources."datasources\\.yaml".datasources[0].url=http://prometheus-server \
    --set datasources."datasources\\.yaml".datasources[0].access=proxy \
    --set datasources."datasources\\.yaml".datasources[0].isDefault=true

# Apply Grafana dashboards
echo "📋 Applying Grafana dashboards..."
kubectl apply -f "$INFRA_DIR/monitoring/grafana-dashboards.yaml"
kubectl apply -f "$INFRA_DIR/monitoring/grafana-datasources.yaml"

# Wait for pods to be ready
echo "⏳ Waiting for monitoring stack to be ready..."
kubectl wait --for=condition=ready pod -l app=prometheus -n "$NAMESPACE" --timeout=120s || true
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=grafana -n "$NAMESPACE" --timeout=120s || true

# Get Grafana credentials
echo ""
echo "🎉 Monitoring stack installed!"
echo ""
echo "📊 Grafana:"
echo "  URL: http://localhost:3000 (run: kubectl port-forward svc/grafana 3000:80 -n $NAMESPACE)"
echo "  Username: admin"
echo "  Password: admin (or get from: kubectl get secret grafana -n $NAMESPACE -o jsonpath='{.data.admin-password}' | base64 -d)"
echo ""
echo "🔥 Prometheus:"
echo "  URL: http://localhost:9090 (run: kubectl port-forward svc/prometheus-server 9090:80 -n $NAMESPACE)"
