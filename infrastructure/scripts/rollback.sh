#!/bin/bash
#
# Rollback script for RustChat Kubernetes deployment
#

set -e

NAMESPACE="rustchat"
DEPLOYMENT="${1:-}"
REVISION="${2:-}"

if [[ -z "$DEPLOYMENT" ]]; then
    echo "Usage: $0 <deployment-name> [revision]"
    echo ""
    echo "Available deployments:"
    kubectl get deployments -n "$NAMESPACE" -o name | sed 's/deployment.apps\//  /'
    echo ""
    echo "To see revision history:"
    echo "  kubectl rollout history deployment/<name> -n $NAMESPACE"
    exit 1
fi

echo "📜 Rollout history for $DEPLOYMENT:"
kubectl rollout history "deployment/$DEPLOYMENT" -n "$NAMESPACE"

if [[ -n "$REVISION" ]]; then
    echo ""
    echo "⏪ Rolling back to revision $REVISION..."
    kubectl rollout undo "deployment/$DEPLOYMENT" -n "$NAMESPACE" --to-revision="$REVISION"
else
    echo ""
    echo "⏪ Rolling back to previous revision..."
    kubectl rollout undo "deployment/$DEPLOYMENT" -n "$NAMESPACE"
fi

echo ""
echo "⏳ Waiting for rollback to complete..."
kubectl rollout status "deployment/$DEPLOYMENT" -n "$NAMESPACE"

echo ""
echo "✅ Rollback complete!"
kubectl get pods -n "$NAMESPACE"
