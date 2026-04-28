#!/bin/bash
set -e

# SAATHI Scope Verification Script
# Tests that AOM and Faculty users can only access their assigned schools.
#
# Usage:
#   1. Start the backend: cd backend && cargo run
#   2. In another terminal: bash scripts/verify-scoping.sh

API="http://localhost:3000/api"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

failures=0

# ── helpers ──────────────────────────────────────────────────────────────────

login() {
  local user="$1"
  local pass="$2"
  curl -s -X POST "$API/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$user\",\"password\":\"$pass\"}" | jq -r '.token'
}

expect_ok() {
  local code="$1"
  local desc="$2"
  if [ "$code" -ge 200 ] && [ "$code" -lt 300 ]; then
    echo -e "${GREEN}✓${NC} $desc (HTTP $code)"
  else
    echo -e "${RED}✗${NC} $desc (HTTP $code, expected 2xx)"
    ((failures++))
  fi
}

expect_forbidden() {
  local code="$1"
  local desc="$2"
  if [ "$code" -eq 403 ]; then
    echo -e "${GREEN}✓${NC} $desc (HTTP $code)"
  else
    echo -e "${RED}✗${NC} $desc (HTTP $code, expected 403)"
    ((failures++))
  fi
}

call() {
  local token="$1"
  local method="$2"
  local path="$3"
  local body="${4:-}"
  if [ -n "$body" ]; then
    curl -s -o /dev/null -w "%{http_code}" -X "$method" "$API$path" \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer $token" \
      -d "$body"
  else
    curl -s -o /dev/null -w "%{http_code}" -X "$method" "$API$path" \
      -H "Authorization: Bearer $token"
  fi
}

count_json() {
  local token="$1"
  local path="$2"
  curl -s -X GET "$API$path" -H "Authorization: Bearer $token" | jq 'length'
}

# ── login all personas ───────────────────────────────────────────────────────

echo "Logging in test users..."
ADMIN_TOKEN=$(login "admin" "admin123")
AOM_TOKEN=$(login "aom1" "aom123")
FACULTY_TOKEN=$(login "faculty1" "faculty123")
VIEWER_TOKEN=$(login "viewer1" "viewer123")

if [ "$ADMIN_TOKEN" = "null" ] || [ -z "$ADMIN_TOKEN" ]; then
  echo -e "${RED}Failed to login as admin. Is the backend running on localhost:3000?${NC}"
  exit 1
fi

echo ""
echo "=== Admin (unscoped) ==="
ADMIN_SCHOOLS=$(count_json "$ADMIN_TOKEN" "/schools")
echo "Admin sees $ADMIN_SCHOOLS schools"
expect_ok 200 "Admin can list schools"

echo ""
echo "=== AOM (scoped to school 1: Green Valley) ==="
AOM_SCHOOLS=$(count_json "$AOM_TOKEN" "/schools")
echo "AOM sees $AOM_SCHOOLS schools"
if [ "$AOM_SCHOOLS" -eq 1 ]; then
  echo -e "${GREEN}✓${NC} AOM sees exactly 1 school"
else
  echo -e "${RED}✗${NC} AOM sees $AOM_SCHOOLS schools (expected 1)"
  ((failures++))
fi

# AOM should be able to create a student in their school
code=$(call "$AOM_TOKEN" POST "/students" '{"school_id":1,"name":"Test Student","grade_level":"Grade 6","program_track":"Foundation","track":""}')
expect_ok "$code" "AOM can create student in assigned school"

# AOM should NOT be able to create a student in another school
code=$(call "$AOM_TOKEN" POST "/students" '{"school_id":2,"name":"Test Student 2","grade_level":"Grade 6","program_track":"Foundation","track":""}')
expect_forbidden "$code" "AOM blocked from creating student in unassigned school"

# AOM should NOT be able to drop an unassigned school
code=$(call "$AOM_TOKEN" POST "/schools/2/drop" '{"reason":"test"}')
expect_forbidden "$code" "AOM blocked from dropping unassigned school"

echo ""
echo "=== Faculty (scoped to school 2: North City) ==="
FAC_SCHOOLS=$(count_json "$FACULTY_TOKEN" "/schools")
echo "Faculty sees $FAC_SCHOOLS schools"
if [ "$FAC_SCHOOLS" -eq 1 ]; then
  echo -e "${GREEN}✓${NC} Faculty sees exactly 1 school"
else
  echo -e "${RED}✗${NC} Faculty sees $FAC_SCHOOLS schools (expected 1)"
  ((failures++))
fi

# Faculty should NOT be able to create a faculty assignment in another school
code=$(call "$FACULTY_TOKEN" POST "/faculty-assignments" '{"faculty_user_id":1,"school_id":1,"grade_level":"Grade 6","track":"","subject_id":1}')
expect_forbidden "$code" "Faculty blocked from assignment in unassigned school"

echo ""
echo "=== Viewer (read-only, unscoped) ==="
VIEW_SCHOOLS=$(count_json "$VIEWER_TOKEN" "/schools")
echo "Viewer sees $VIEW_SCHOOLS schools"
if [ "$VIEW_SCHOOLS" -eq "$ADMIN_SCHOOLS" ]; then
  echo -e "${GREEN}✓${NC} Viewer sees all schools (unscoped read-only)"
else
  echo -e "${RED}✗${NC} Viewer sees $VIEW_SCHOOLS schools (expected $ADMIN_SCHOOLS)"
  ((failures++))
fi

# Viewer should NOT be able to create a student (no write access)
code=$(call "$VIEWER_TOKEN" POST "/students" '{"school_id":1,"name":"Viewer Student","grade_level":"Grade 6","program_track":"Foundation","track":""}')
expect_forbidden "$code" "Viewer blocked from creating student"

echo ""
echo "================================"
if [ "$failures" -eq 0 ]; then
  echo -e "${GREEN}All scope checks passed!${NC}"
  exit 0
else
  echo -e "${RED}$failures check(s) failed.${NC}"
  exit 1
fi
