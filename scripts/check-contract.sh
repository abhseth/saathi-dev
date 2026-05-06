#!/usr/bin/env bash
# check-contract.sh — Verify frontend API calls match the dispatch table in api.ts
set -euo pipefail
export LC_ALL=C

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
API_TS="$PROJECT_ROOT/frontend/src/api.ts"
SRC_DIR="$PROJECT_ROOT/frontend/src"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ERRORS=0

# ── Extract dispatch keys from api.ts ─────────────────────────────────────────
# Matches "command_name: {" inside the dispatch object
DISPATCH_KEYS=$(grep -E '^\s+[a-z_]+:\s*\{' "$API_TS" | sed -E 's/^[[:space:]]+([a-z_]+):.*/\1/' | grep -vx 'headers' | sort -u)
DISPATCH_COUNT=$(echo "$DISPATCH_KEYS" | wc -l)

echo "=== Contract Check ==="
echo "Found $DISPATCH_COUNT commands in dispatch table"

# ── Extract all api("...") calls from frontend source ────────────────────────
# Extract api("cmd" calls, skipping lines that are comments
# Use perl to match api("..." while checking the line doesn't start with //
API_CALLS=$(find "$SRC_DIR" -name '*.ts' -o -name '*.tsx' | xargs perl -nle 'if (m/^\s*\/\//) { next } if (m/api\b[^"]*"([a-z_]+)"/) { print $1 }' | grep -vE '^(command_name|headers)$' | sort -u)
API_CALL_COUNT=$(echo "$API_CALLS" | wc -l)

echo "Found $API_CALL_COUNT unique api() calls in source"

# ── Check 1: Every api() call must exist in dispatch ──────────────────────────
echo ""
echo "--- Checking api() calls against dispatch table ---"
MISSING_FROM_DISPATCH=0
while IFS= read -r cmd; do
  [ -z "$cmd" ] && continue
  if ! echo "$DISPATCH_KEYS" | grep -qx "$cmd"; then
    echo -e "${RED}ERROR:${NC} api() calls unknown command: ${YELLOW}$cmd${NC}"
    MISSING_FROM_DISPATCH=$((MISSING_FROM_DISPATCH + 1))
  fi
done <<< "$API_CALLS"

if [ "$MISSING_FROM_DISPATCH" -eq 0 ]; then
  echo -e "${GREEN}✓${NC} All api() calls reference valid dispatch commands"
else
  ERRORS=$((ERRORS + MISSING_FROM_DISPATCH))
fi

# ── Check 2: Every dispatch key should be referenced somewhere ────────────────
echo ""
echo "--- Checking dispatch coverage ---"
ORPHAN_COMMANDS=0
while IFS= read -r cmd; do
  [ -z "$cmd" ] && continue
  # Skip commands that are only called via string interpolation or dynamic names
  if ! echo "$API_CALLS" | grep -qx "$cmd"; then
    # Check if it's referenced in a helper function in api.ts itself
    if grep -qE "\"$cmd\"|'$cmd'" "$API_TS"; then
      continue
    fi
    echo -e "${YELLOW}WARN:${NC} Dispatch command not referenced in frontend: ${YELLOW}$cmd${NC}"
    ORPHAN_COMMANDS=$((ORPHAN_COMMANDS + 1))
  fi
done <<< "$DISPATCH_KEYS"

if [ "$ORPHAN_COMMANDS" -eq 0 ]; then
  echo -e "${GREEN}✓${NC} All dispatch commands are referenced in frontend code"
else
  # Orphans are warnings, not hard errors — some endpoints may be reserved for future use
  echo "  ($ORPHAN_COMMANDS orphan commands — consider removing if truly unused)"
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
if [ "$ERRORS" -eq 0 ]; then
  echo -e "${GREEN}=== Contract check passed ===${NC}"
  exit 0
else
  echo -e "${RED}=== Contract check failed with $ERRORS error(s) ===${NC}"
  exit 1
fi
