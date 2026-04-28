# Strategy: Moving to Production-Ready Multi-School Scoping

This document outlines the deep technical strategy for completing Phase 1 (Steps 6-8) and addressing the architectural debt identified in the `REVIEW.md`.

## 1. Architectural Upgrade: Concurrency & Persistence

Before scaling to multiple AOM users, the following infrastructure changes are prioritized:

### A. Connection Pooling (`r2d2`)
The current `Mutex<Connection>` is a serialization point.
- **Action:** Replace `rusqlite` with `r2d2_sqlite`.
- **Impact:** Allows the Axum server to handle multiple simultaneous DB-heavy requests (e.g., multiple AOMs loading dashboards) without blocking each other.

### B. Persistence Assurance
- **Action:** Configure Railway/Fly.io Volumes to mount at `/data`.
- **Action:** Update `DATABASE_PATH` environment variable to `/data/tickets.sqlite3`.
- **Goal:** Eliminate the need for manual DB snapshots and prevent data loss on deployment.

---

## 2. Secure Scope Enforcement (Step 6)

The primary goal is ensuring AOMs and Faculty only interact with schools assigned to them in the `user_schools` table.

### Phase A: JWT Enrichment
- **Mechanism:** Modify `auth::issue_token` to include `school_ids` in the `Claims`.
- **Benefit:** Most scope checks (e.g., "Does this user have access to School ID 5?") can be performed in-memory by the middleware or handler without querying the database.

### Phase B: Repository Refactoring
Listing functions must be updated to support optional filtering:
- `list_tickets(conn, school_ids: Option<&[i64]>)`
- `list_students(conn, school_ids: Option<&[i64]>)`
- `list_timetable_slots(conn, school_ids: Option<&[i64]>)`

If `school_ids` is `Some`, the SQL query will append `WHERE school_id IN (...)`. If `None` (Admin role), the query returns all.

### Phase C: Handler Logic (The "Enforcer")
A helper function `enforce_school_scope(claims: &Claims, school_id: i64)` will be used:
1. If `claims.role == "admin"`, return `Ok`.
2. If `claims.school_ids.contains(&school_id)`, return `Ok`.
3. Otherwise, return `Err(AppError::forbidden("Access to this school is denied"))`.

This must be called at the start of:
- `create_ticket`, `update_ticket`
- `create_student`
- `upsert_timetable_slot`, `set_school_optional_subject`

---

## 3. Data Integrity & Validation

### "Ghost" School IDs
We must ensure that an AOM cannot "guess" a school ID and assign it to a student or ticket. 
- **Validation:** Every mutation input containing a `school_id` must be validated against the user's `Claims.school_ids`.

### The Student-School Link
When creating a ticket for a student, the system must verify that the `student_id` actually belongs to a `school_id` the user has access to.

---

## 4. Implementation Roadmap for Next Sessions

### Turn 1: Infrastructure & Models
1. Update `models.rs` (Claims/CurrentUser) to include `school_ids`.
2. Update `repositories.rs` (`authenticate_user`) to fetch those IDs.
3. Update `auth.rs` (`issue_token`) to sign the token with those IDs.

### Turn 2: Scoped Repositories
1. Modify repository listing functions to accept the `school_ids` filter.
2. Update SQL queries to use `IN` clauses for filtering.

### Turn 3: Handler Enforcement
1. Add `enforce_school_scope` to `auth.rs` or a utility module.
2. Audit every route in `routes/` and apply the check or filter.

### Turn 4: Validation & Step 7
1. Implement the UI toggles for English/SST opt-ins (Step 7).
2. Create an automated test script (Bash/Curl) that attempts cross-school access to verify the fix.
