#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== SAATHI CI Gate ==="

# Backend
echo ""
echo "--- Backend: cargo test ---"
cd "$PROJECT_ROOT/backend"
cargo test --quiet

# Contract check
echo ""
echo "--- Contract check ---"
cd "$PROJECT_ROOT"
bash scripts/check-contract.sh

# Frontend tests
echo ""
echo "--- Frontend: npm run test ---"
cd "$PROJECT_ROOT/frontend"
npm run test

# Frontend typecheck
echo ""
echo "--- Frontend: npm run typecheck ---"
cd "$PROJECT_ROOT/frontend"
npm run typecheck

# Frontend build
echo ""
echo "--- Frontend: npm run build ---"
cd "$PROJECT_ROOT/frontend"
npm run build

echo ""
echo "=== All gates passed ==="
