# SAATHI Architecture & Code Quality Review

**Review Date:** 2026-04-30  
**Scope:** Backend (`src/`) + Frontend (`src/`) — 14 core source files  
**Criteria:** Modularity, Error Handling, Performance, Security, Data Integrity, Frontend Architecture, Testing/Observability  

---

## Executive Summary

SAATHI is a functionally rich, six-phase monolith that successfully delivers ticket management, school master data, faculty timetabling, attendance tracking, substitution orchestration, and analytics dashboards. The codebase is **operationally effective** but carries significant **structural debt** from rapid iterative development. The most critical risks are a **7,200-line god repository file**, **SQL injection vectors** in dynamic `IN` clauses, and a **3,100-line frontend root component** that centralizes all state. The good news: the route layer is thin and well-factored, authentication is solid (JWT + bcrypt + rate limiting), migrations follow a data-preserving discipline, and the Tauri-to-HTTP migration abstraction (`api.ts`) is clean. Addressing the three critical issues below should be the immediate priority before adding Phase 7 features.

---

## Critical Findings 🔴

### 1. `repositories.rs` is a 7,213-line god file
**Location:** `backend/src/repositories.rs`  
**Impact:** Maintainability, compile times, code ownership, review burden  

The entire data access layer — tickets, comments, history, schools, students, regions, users, subjects, faculty assignments, timetable slots, weekly slots, lecture sessions, attendance, holidays, compliance metrics, substitution records, leave/swap requests, and health analytics — lives in a single file. Business logic (escalation refresh, audit logging, linked metadata extraction, conflict detection) is mixed with raw SQL, row mapping, and validation constants.

**Evidence:**
- `ALLOWED_PRIORITIES`, `ALLOWED_STATUSES`, `ALLOWED_QUEUES` const slices defined at module level.
- `refresh_escalations()` evaluates SLA risk and updates tickets inline.
- `create_ticket()` calls `record_history()` 13+ times interleaved with SQL.
- Row mappers (`ticket_from_row`, `school_from_row`, etc.) occupy hundreds of lines each.

**Recommended fix:** Split into domain modules:
```
src/repositories/
  mod.rs          // shared helpers (record_history, record_audit, validate_*)
  tickets.rs      // ticket + comment + history + attachment
  schools.rs      // school + region + student + batch
  users.rs        // user + auth + user_schools
  timetable.rs    // template slots + weekly slots + holidays
  faculty.rs      // assignments + lecture sessions + attendance
  substitution.rs // leave + swap + suggestions + balance
  analytics.rs    // health + compliance + deviation (read-only aggregations)
```

---

### 2. SQL injection via unsanitized `IN` clause interpolation
**Location:** `backend/src/repositories.rs`, `backend/src/analytics.rs`, `backend/src/alerts.rs`  
**Impact:** Security — potential data exfiltration or unauthorized scope bypass  
**Severity: HIGH**

Dozens of functions build dynamic SQL using `format!("… IN ({list})")` where `list` is constructed by `.map(|i| i.to_string()).collect::<Vec<_>>().join(",")`. While the values come from the `scope_filter()` helper (which derives from JWT claims), this pattern bypasses SQLite parameter binding and is brittle. Any future refactor that feeds user input into these vectors creates an immediate injection vector.

**Evidence:**
```rust
// repositories.rs ~line 1475
let list = ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",");
sql.push_str(&format!(" AND students.school_id IN ({list})"));
```

Similar patterns exist in:
- `list_tickets`, `list_schools`, `list_dropped_schools`, `list_students`, `list_batches`
- `list_faculty_assignments`, `list_timetable_slots`, `list_weekly_timetable_slots`
- `list_leave_requests`, `list_substitution_records`, `list_holidays`
- `analytics::compliance_scorecard`, `deviation_scoreboard`, `faculty_utilization_trend`
- `alerts::check_unfilled_periods`, `check_double_bookings`, etc.

**Recommended fix:** Use `rusqlite` parameter binding for `IN` clauses. Build placeholder strings (`?1,?2,?3`) and pass the IDs through `rusqlite::params_from_iter()`. Some functions already do this correctly (e.g., `list_faculty_today_sessions`, `attendance_summary`), proving the pattern works.

---

### 3. `App.tsx` is a 3,100-line god component with 60+ `useState` hooks
**Location:** `frontend/src/App.tsx`  
**Impact:** Frontend maintainability, state synchronization bugs, testability  

Every piece of global state — tickets, comments, schools, students, faculty, timetable slots, weekly slots, analytics data, substitution records, leave requests, swap requests, policies, escalation rules, alerts, announcements, UI flags — is declared via `useState` in a single component. Data loaders are fired shotgun-style in a `useEffect` on mount. The component implements a monolithic `adminContent` ternary tree with 40+ panel branches.

**Evidence:**
```tsx
const [tickets, setTickets] = React.useState<Ticket[]>([]);
// ... 60+ more state variables

React.useEffect(() => {
  void loadTickets(); void loadAssignmentRules(); void loadSchools();
  // ~15 loaders fired simultaneously
}, []);
```

**Recommended fix:** Introduce lightweight global state management:
- **Zustand** or **Jotai** for global data stores (tickets, schools, users).
- **React Query / TanStack Query** for server-state caching, background refetching, and deduplication.
- Extract domain hooks: `useTickets()`, `useSchools()`, `useTimetable()`, `useAnalytics()`.
- Keep `App.tsx` as a routing/layout shell only.

---

## Warnings 🟡

### 4. Error type erosion: `Result<T, String>` everywhere
**Location:** `backend/src/repositories.rs`, `backend/src/analytics.rs`, `backend/src/alerts.rs`, `backend/src/bulk_ops.rs`, etc.  
**Impact:** Brittle error handling, inability to return semantic HTTP status codes

Every repository function returns `Result<T, String>`. The `AppError` type in `error.rs` is well-designed (`bad_request`, `unauthorized`, `forbidden`, `not_found`, `internal`) but its `From<String>` implementation unconditionally maps to `INTERNAL_SERVER_ERROR` with a generic message. Route handlers manually map some strings (`AppError::bad_request(e)`) but most repository errors surface as 500s even when they represent validation failures or not-found conditions.

**Evidence:**
```rust
// error.rs
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        tracing::error!("Internal error: {}", msg);
        Self::internal("An internal error occurred")
    }
}
```

**Recommended fix:** Define a structured `RepoError` enum:
```rust
pub enum RepoError {
    NotFound(&'static str, i64),
    Validation(&'static str),
    Conflict(&'static str),
    Database(rusqlite::Error),
}
```
Implement `From<RepoError> for AppError` so `NotFound` → 404, `Validation` → 400, `Conflict` → 409, `Database` → 500. This allows route handlers to use `?` uniformly while returning correct status codes.

---

### 5. N+1 query patterns in analytics and list operations
**Location:** `backend/src/analytics.rs`, `backend/src/repositories.rs`  
**Impact:** Performance degradation at scale

- `faculty_utilization_trend()` iterates faculty list × weeks, executing a `COUNT(*)` query per iteration.
- `list_users()` fetches all users, then loops to call `list_user_schools()` per user (N+1).
- `attendance_summary()` uses correlated subqueries for present/late/absent/excused counts instead of a single aggregation.
- `get_deviation_score()` runs multiple independent queries that could be batched.

**Recommended fix:** Replace looped queries with SQL `GROUP BY` aggregations or batched `IN` lookups (using proper parameter binding). For `list_users`, use a single join query or a batched school-id fetch.

---

### 6. Alert cache is in-process only and lacks invalidation
**Location:** `backend/src/alerts.rs`  
**Impact:** Stale alerts under horizontal scaling; cache inconsistency

```rust
static ALERT_CACHE: LazyLock<Mutex<HashMap<String, (Vec<Alert>, Instant)>>> = ...;
```

The 30-second cache is stored in a global `HashMap` keyed by comma-joined school IDs. If SAATHI ever runs behind multiple processes (e.g., Fly.io with multiple VMs), each process maintains its own cache, leading to inconsistent alert states. There is also no cache invalidation on data mutation.

**Recommended fix:** Either remove the cache (SQLite is fast for read-only analytical queries) or replace with a time-bounded per-request memoization. If caching remains necessary, use Redis or a shared cache.

---

### 7. Naive CSV parsing in bulk import
**Location:** `backend/src/bulk_ops.rs`  
**Impact:** Data corruption on quoted fields containing commas

```rust
let cols: Vec<&str> = line.split(',').collect();
```

This will break on any quoted CSV field containing a comma (e.g., `"Shah, Priya"`).

**Recommended fix:** Use the `csv` crate for robust parsing.

---

### 8. `count_school_field` accepts a raw column name
**Location:** `backend/src/repositories.rs` ~line 2465  
**Impact:** Potential SQL injection if caller is ever exposed to user input

```rust
fn count_school_field(conn: &Connection, field_name: &str, value: &str) -> Result<i64, String> {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM schools WHERE is_dropped = 0 AND {field_name} = ?1"),
        params![value],
        |row| row.get(0),
    )
```

While currently called only with hardcoded column names, this is a latent risk.

**Recommended fix:** Maintain an allowlist of valid column names and reject anything not in the list.

---

### 9. Migration debt: table rebuilds for constraint changes
**Location:** `backend/src/db.rs` migrations 22, 39, 40, 43  
**Impact:** Operational risk during deployment

Migrations 39 and 40 stripped and restored foreign keys on `lecture_sessions` via table recreation. Migration 43 recreates `timetable_slots` and `timetable_weekly_slots` to add `CHECK` constraints. These are large, blocking operations on a live SQLite file.

**Recommended fix:** Document the migration rollback procedure. For future constraint changes on large tables, consider adding constraints via triggers or application-level validation instead of table rebuilds.

---

### 10. `loadLectureSessions` is a stub TODO
**Location:** `frontend/src/App.tsx` ~line 491  
**Impact:** Non-functional feature in production

```tsx
const loadLectureSessions = React.useCallback(async (_schoolId: number, _gradeLevel: string) => {
  // TODO: Wire up to a real lecture-sessions endpoint when backend adds it.
  setLectureSessions([]);
}, []);
```

This dead code is passed into `TicketDetail` but always returns empty data.

**Recommended fix:** Either implement the backend endpoint (`GET /lecture-sessions?school_id=&grade_level=`) and wire it up, or remove the dead code and the prop drilling.

---

### 11. Frontend type drift via inline dynamic imports
**Location:** `frontend/src/App.tsx`  
**Impact:** Type safety erosion, bundler complexity

```tsx
const [policiesData, setPoliciesData] = React.useState<import("./types").CentralPolicy[]>([]);
```

Types like `CentralPolicy` and `Announcement` are imported dynamically instead of at the top level, suggesting they were added late and not fully integrated into the module graph.

**Recommended fix:** Add static top-level imports for all types used in state declarations.

---

## Observations 🟢

### 12. Thin route layer with good auth middleware
**Location:** `backend/src/routes/*.rs`  
**Assessment:** Positive

Route modules are thin: extract state, get a DB connection, enforce school scope, call repository/analytics, return JSON. The `require_auth` middleware cleanly inserts `Claims` into request extensions. `scope_filter()` and `enforce_school_scope()` provide a consistent authorization pattern. Rate limiting on login (`MAX_ATTEMPTS = 5`, 15-minute lockout) is well-implemented.

---

### 13. Clean Tauri-to-HTTP abstraction
**Location:** `frontend/src/api.ts`  
**Assessment:** Positive

The dispatch table maps legacy Tauri command names to HTTP methods/paths, allowing the rest of the frontend to call `api("command_name", args)` identically to the old `invoke()` API. JWT injection, error extraction, file upload/download, and multipart support are all handled cleanly.

---

### 14. Data-preserving migration discipline
**Location:** `backend/src/db.rs`  
**Assessment:** Positive

Migrations use `column_exists()` and `migration_applied()` guards to be idempotent. Schema changes follow an ADD COLUMN with default, or recreate-and-swap pattern. Seed data is wrapped in `ON CONFLICT … DO NOTHING`. WAL mode and `PRAGMA foreign_keys = ON` are set on open.

---

### 15. Password security
**Location:** `backend/src/repositories.rs` (user CRUD), `backend/src/auth.rs`  
**Assessment:** Positive

Passwords are hashed with `bcrypt` at default cost. The `change_password` flow verifies the current hash before updating. Login returns a generic "Invalid username or password" message to prevent user enumeration. JWT uses HS256 with an 8-hour expiry.

---

### 16. Status transition validation
**Location:** `backend/src/repositories.rs` ~line 2506  
**Assessment:** Positive

Ticket status transitions are explicitly validated:
```rust
"Closed" => &["Open"], // reopen only
```
This prevents invalid workflow jumps.

---

## Recommendations (Prioritized)

| Priority | Action | Effort | File(s) |
|----------|--------|--------|---------|
| P0 | Fix SQL injection in all `IN` clauses by switching to parameter binding | Medium | `repositories.rs`, `analytics.rs`, `alerts.rs` |
| P0 | Split `repositories.rs` into domain modules | High | `repositories.rs` → `repositories/{mod,tickets,schools,users,timetable,faculty,substitution,analytics}.rs` |
| P0 | Introduce frontend state management (Zustand / TanStack Query) and break up `App.tsx` | High | `frontend/src/App.tsx`, new `stores/`, `hooks/` |
| P1 | Replace `Result<T, String>` with structured `RepoError` enum | Medium | `repositories.rs`, `error.rs` |
| P1 | Eliminate N+1 queries in analytics and `list_users` | Medium | `analytics.rs`, `repositories.rs` |
| P1 | Replace naive CSV `split(',')` with `csv` crate | Low | `bulk_ops.rs` |
| P2 | Remove or implement `loadLectureSessions` stub | Low | `App.tsx` |
| P2 | Add static imports for `CentralPolicy`, `Announcement` types | Low | `App.tsx` |
| P2 | Sanitize `count_school_field` with column allowlist | Low | `repositories.rs` |
| P2 | Remove in-process alert cache or replace with per-request memoization | Low | `alerts.rs` |
| P3 | Add integration tests for route handlers | High | `tests/` or `routes/*_test.rs` |
| P3 | Add structured tracing spans to repository functions | Low | `repositories.rs` |

---

*Review prepared for SAATHI maintainers.*
