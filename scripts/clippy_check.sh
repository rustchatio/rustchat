#!/bin/bash
# Clippy validation script for rustchat backend
# Runs clippy with strict warnings-as-errors mode

set -euo pipefail

cd "$(dirname "$0")/../backend"

echo "Running cargo clippy with -D warnings..."
cargo clippy --all-targets --all-features -- -D warnings

echo "✅ Clippy check passed!"
