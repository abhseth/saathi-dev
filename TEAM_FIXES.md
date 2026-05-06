# Team Fix Report — 5 Critical Security Issues

**Date:** 2026-04-30  
**Team Size:** 5 specialist agents  
**Coordination Model:** All 5 fixes were independent (no file overlap), executed in parallel  
**Build Status:** ✅ Backend compiles (`cargo build` passes) · ✅ Frontend compiles (`npm run build` passes)

---

## Agent Roster & Assignments

| Agent | Codename | Fix | Files Modified | Status |
|-------|----------|-----|---------------|--------|
| 1 | **JWT Sentinel** | Remove hardcoded JWT fallback secret | `backend/src/main.rs` | ✅ Done |
| 2 | **Seed Gatekeeper** | Gate test user seeding behind env var | `backend/src/db.rs` | ✅ Done |
| 3 | **Scope Enforcer** | Fix scope bypass via `school_name` | `backend/src/routes/tickets.rs`, `backend/src/repositories.rs` | ✅ Done |
| 4 | **Rate Limit Guardian** | Add brute-force protection to login | `backend/src/routes/auth.rs` | ✅ Done |
| 5 | **CSV Shield** | Prevent CSV formula injection | `backend/src/routes/export.rs` | ✅ Done |

---

## Fix 1: JWT Sentinel — Remove Hardcoded Fallback Secret

**Agent:** Agent 1  
**File:** `backend/src/main.rs`  
**Severity:** Critical

### Before
```rust
let jwt_secret = std::env::var("JWT_SECRET")
    .unwrap_or_else(|_| "dev-secret-change-in-production".to_string());

if jwt_secret.len() < 32 {
    tracing::warn!("JWT_SECRET is short — use a 32+ character secret in production");
}
```

### After
```rust
let jwt_secret = std::env::var("JWT_SECRET")
    .expect("JWT_SECRET environment variable must be set");

if jwt_secret.len() < 32 {
    panic!("JWT_SECRET must be at least 32 characters");
}
```

**Impact:** The application now aborts startup if `JWT_SECRET` is missing or shorter than 32 characters. No fallback token can be forged.

---

## Fix 2: Seed Gatekeeper — Gate Test User Seeding

**Agent:** Agent 2  
**File:** `backend/src/db.rs`  
**Severity:** Critical

### Before
```rust
seed_test_users(conn)?;
```

### After
```rust
if std::env::var("TEST_SEED").unwrap_or_default() == "1" {
    seed_test_users(conn)?;
}
```

**Impact:** Test users (`aom1`/`aom123`, `faculty1`/`faculty123`, etc.) are only created when `TEST_SEED=1` is explicitly set. Production deployments are safe by default.

**Operational Note:** For development environments, set `TEST_SEED=1` before starting the backend.

---

## Fix 3: Scope Enforcer — Fix Horizontal Privilege Escalation

**Agent:** Agent 3  
**Files:** `backend/src/routes/tickets.rs`, `backend/src/repositories.rs`  
**Severity:** High

**Vulnerability:** An AOM could send `school_id: null` (or omit it) and provide `school_name: "Victim School"`. The route skipped `enforce_school_scope` because `school_id` was `None`, but the repository resolved `school_name` to a real ID later — creating/updating tickets for schools outside their scope.

### Fix in `repositories.rs`
Made `resolve_ticket_school` public:
```rust
pub fn resolve_ticket_school(...)  // was `fn resolve_ticket_school(...)`
```

### Fix in `routes/tickets.rs` — `create_ticket`
```rust
let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
let (resolved_school_id, _) = repositories::resolve_ticket_school(&*conn, input.school_id, &input.school_name)
    .map_err(|e| AppError::bad_request(e))?;
if let Some(sid) = resolved_school_id {
    enforce_school_scope(&claims, sid)?;
}
Ok(Json(repositories::create_ticket(&*conn, &input, &claims.display_name)?))
```

### Fix in `routes/tickets.rs` — `update_ticket`
```rust
let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
let existing = repositories::get_ticket(&*conn, id)?;
if let Some(sid) = existing.school_id {
    enforce_school_scope(&claims, sid)?;  // enforce on existing ticket
}
let (resolved_school_id, _) = repositories::resolve_ticket_school(&*conn, input.school_id, &input.school_name)
    .map_err(|e| AppError::bad_request(e))?;
if let Some(sid) = resolved_school_id {
    enforce_school_scope(&claims, sid)?;  // enforce on new school
}
Ok(Json(repositories::update_ticket(&*conn, &input, &claims.display_name)?))
```

**Impact:** Scope enforcement now happens on the **resolved** school ID, not just the input ID. The `school_name` bypass is closed.

---

## Fix 4: Rate Limit Guardian — Brute-Force Protection

**Agent:** Agent 4  
**File:** `backend/src/routes/auth.rs`  
**Severity:** High

### Implementation
Added a module-level in-memory rate limiter using `std::sync::LazyLock` (no new dependencies):

```rust
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

static LOGIN_ATTEMPTS: LazyLock<Mutex<HashMap<String, (u32, Instant)>>> = 
    LazyLock::new(|| Mutex::new(HashMap::new()));

const MAX_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION: Duration = Duration::from_secs(15 * 60); // 15 minutes
```

**Behavior:**
- Before authenticating: check if username has ≥5 failed attempts within 15 minutes → return 429
- On failed auth: increment counter, update timestamp
- On successful auth: clear counter
- Expired entries (older than 15 min) are cleaned up on each check

**Impact:** Brute-force attacks against login are throttled. Even with weak test passwords, an attacker cannot try more than 5 guesses per 15-minute window per username.

---

## Fix 5: CSV Shield — Formula Injection Prevention

**Agent:** Agent 5  
**File:** `backend/src/routes/export.rs`  
**Severity:** High

### Before
```rust
fn field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
```

### After
```rust
fn field(s: &str) -> String {
    let mut result = if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    };
    // Defuse formula injection: prefix trigger characters with a single quote
    if let Some(first) = result.chars().next() {
        if first == '=' || first == '+' || first == '-' || first == '@' || first == '\t' {
            result.insert(0, '\'');
        }
    }
    result
}
```

**Impact:** Exported CSV fields that start with formula trigger characters (`=`, `+`, `-`, `@`, tab) are prefixed with a single quote, neutralizing Excel/Sheets formula execution. A ticket description like `=cmd|' /C calc'!A0` becomes `'=cmd|' /C calc'!A0` in the export.

---

## Coordination Notes

### Parallel Execution Strategy
All 5 fixes were fully independent:
- **Agent 1** touched `main.rs` (JWT initialization)
- **Agent 2** touched `db.rs` (seed gating)
- **Agent 3** touched `tickets.rs` + `repositories.rs` (scope enforcement)
- **Agent 4** touched `auth.rs` (module-level static, no main.rs touch needed)
- **Agent 5** touched `export.rs` (field sanitization)

No file overlaps → all 5 agents launched simultaneously. No conflicts arose.

### Manager Verification Steps
1. Reviewed all agent reports for completeness
2. Ran `cargo check` after each agent reported back
3. Ran `cargo build` for final full compilation verification
4. Verified `npm run build` on frontend (unchanged, still passes)
5. Confirmed no unintended side effects in modified files

---

## Remaining Security Items (Not P0 — Defer to Next Sprint)

Per the Council Review, these are important but not deployment-blockers:

| Item | Severity | Notes |
|------|----------|-------|
| Add secondary indexes | High | Migration 40 recommended |
| Restore FKs on `lecture_sessions` | High | Migration 40 recommended |
| Enforce ticket state machine | High | Workflow gap |
| Add calendar conflict checks | High | Substitution/makeup gaps |
| Protect timetable deletion cascade | High | Historical attendance at risk |
| Add pagination to list endpoints | Medium | DoS/OOM risk at scale |
| Add DB transactions | Medium | Partial write corruption |
| Restrict CORS | Medium | Currently `Any` origin |

---

*End of Team Fix Report*
