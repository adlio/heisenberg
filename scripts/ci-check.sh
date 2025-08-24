#!/bin/bash
set -euo pipefail

echo "🔍 Running CI quality checks..."

echo "📝 Checking code formatting..."
cargo fmt --check

echo "🔧 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🧪 Running tests..."
cargo nextest run

echo "✅ All checks passed!"
