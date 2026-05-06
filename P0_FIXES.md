# P0 Fix Report — Post Follow-Up Review

**Date:** 2026-04-30  
**Team:** 3 specialist agents (DB Architect, Security Patch, Backend Engineer)  
**Coordination:** 2-phase execution (Phase 1 parallel → Phase 2 sequential)  
**Build Status:** ✅ Backend compiles (`cargo build` passes) · ✅ Frontend compiles (`npm run build` passes)

---

## Agent Roster & Assignments

| Phase | Agent | Fix | Files Modified | Status |
|-------|-------|-----|---------------|--------|
| 1 | **DB Architect** | Migration 40 (restore FKs) + Migration 41 (soft-delete) + 13 indexes | `backend/src/db.rs` | ✅ Done |
| 1 | **Security Patch** | Sanitize 500s + lock admin reads + harden CORS | `backend/src/error.rs`, `routes/admin.rs`, `main.rs` | ✅ Done |
| 2 | **Backend Engineer** | Soft-delete timetable slots + LIMIT on 12 list queries | `backend/src/repositories.rs`, `models.rs` | ✅ Done |

---

## Fix 1: DB Architect — Migration 40 + Migration 41 + Indexes

### Migration 40: Restore FKs on `lecture_sessions`

Migration 39 stripped all FKs from `lecture_sessions`. Migration 40 recreates the table with full referential integrity:

- `timetable_slot_id` → `timetable_slots(id)` ON DELETE CASCADE
- `actual_faculty_user_id` → `users(id)` ON DELETE SET NULL
- `subject_id` → `subjects(id)` ON DELETE SET NULL
- `school_id` → `schools(id)` ON DELETE CASCADE

Also cleans orphaned rows before recreation and rebuilds 5 indexes.

### Migration 41: Soft-Delete Support for `timetable_slots`

Adds `deleted_at TEXT` column to `timetable_slots` with a partial index:
```sql
ALTER TABLE timetable_slots ADD COLUMN deleted_at TEXT;
CREATE INDEX idx_timetable_slots_deleted ON timetable_slots(deleted_at) WHERE deleted_at IS NOT NULL;
```

### 13 Critical Indexes Created on Startup

| Index | Table | Columns |
|-------|-------|---------|
| idx_tickets_school | tickets | school_id |
| idx_tickets_updated | tickets | updated_at |
| idx_tickets_escalation | tickets | escalation_status, sla_due_at |
| idx_tickets_status | tickets | status |
| idx_attendance_student | attendance_records | student_id |
| idx_students_school | students | school_id |
| idx_students_batch | students | school_id, grade_level, track, batch_id |
| idx_audit_log_created | audit_log | created_at |
| idx_audit_log_entity | audit_log | entity_type, entity_id |
| idx_ticket_history_ticket | ticket_history | ticket_id |
| idx_ticket_comments_ticket | ticket_comments | ticket_id |
| idx_users_role | users | role |
| idx_faculty_assignments_faculty | faculty_assignments | faculty_user_id |
| idx_faculty_assignments_school | faculty_assignments | school_id |

---

## Fix 2: Security Patch — 3 Issues in 1

### 2A: Sanitize 500 Error Messages (`error.rs`)

**Before:** Raw `String` errors from repositories leaked SQL details, file paths, and schema info to any authenticated user.

**After:**
```rust
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        tracing::error!("Internal error: {}", msg);
        Self::internal("An internal error occurred")
    }
}
```

### 2B: Lock Down 4 Exposed Admin Read Endpoints (`routes/admin.rs`)

Added `require_admin_or_aom(&claims)?` to:
- `GET /sla-policies`
- `GET /assignment-rules`
- `GET /escalation-policy`
- `GET /communication-templates`

These were previously readable by any authenticated user (faculty, viewer, AOM).

### 2C: Harden CORS (`main.rs`)

**Before:**
```rust
.allow_origin(Any)
.allow_methods(Any)
.allow_headers(Any)
```

**After:**
```rust
.allow_origin(
    std::env::var("CORS_ORIGIN")
        .ok()
        .and_then(|s| s.parse::<HeaderValue>().ok())
        .unwrap_or_else(|| "http://localhost:5173".parse().unwrap())
)
.allow_methods([GET, POST, PUT, DELETE])
.allow_headers([CONTENT_TYPE, AUTHORIZATION])
```

Production deployments can set `CORS_ORIGIN=https://your-domain.vercel.app`.

---

## Fix 3: Backend Engineer — Soft-Delete + Pagination

### 3A: Timetable Slot Soft-Delete (`repositories.rs`)

**Before:** Hard `DELETE` with CASCADE destroying all historical `lecture_sessions` and `attendance_records`.

**After:**
```rust
pub fn delete_timetable_slot(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn.execute(
        "UPDATE timetable_slots SET deleted_at = datetime('now', 'localtime') 
         WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
    ).map_err(|e| e.to_string())?;
    // ...
}
```

`list_timetable_slots` and `list_weekly_timetable_slots` now filter `WHERE deleted_at IS NULL`.

### 3B: Safe Limits on 12 List Queries (`repositories.rs`)

Every unbounded list query now caps at **1000 rows** to prevent OOM on the 256MB Fly VM:

| Function | Protected |
|----------|-----------|
| `list_tickets` | ✅ 1000 cap |
| `list_all_comments` | ✅ 1000 cap |
| `list_schools` | ✅ 1000 cap |
| `list_dropped_schools` | ✅ 1000 cap |
| `list_students` | ✅ 1000 cap |
| `list_batches` | ✅ 1000 cap |
| `list_timetable_slots` | ✅ 1000 cap |
| `list_weekly_timetable_slots` | ✅ 1000 cap |
| `list_holidays` | ✅ 1000 cap |
| `list_faculty_assignments` | ✅ 1000 cap |
| `list_subjects` | ✅ 1000 cap |
| `list_users` | ✅ 1000 cap |
| `list_audit_log` | Already had limit |

### 3C: `Paginated<T>` Type Added (`models.rs`)

```rust
#[derive(Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
```

Ready for future endpoint migration to proper pagination.

---

## Coordination Notes

### Phase 1 — Parallel Execution

**DB Architect** and **Security Patch** launched simultaneously because they touched **zero overlapping files**:
- DB Architect → `db.rs` only
- Security Patch → `error.rs` + `routes/admin.rs` + `main.rs`

No conflicts. Both reported `cargo check` clean.

### Phase 2 — Sequential Execution

**Backend Engineer** launched after Phase 1 completed because it needed:
- Migration 41's `deleted_at` column to exist (from DB Architect)
- `repositories.rs` modifications that could conflict with future work

The Backend Engineer modified `repositories.rs` extensively but in a safe manner:
- Soft-delete changes targeted specific functions (`delete_timetable_slot`, `list_timetable_slots`)
- Pagination changes appended `LIMIT ?` to existing ORDER BY clauses
- No function signatures were changed, so no route or frontend code needed updates

### Manager Verification

1. Reviewed all agent reports for completeness
2. Ran `cargo build` for final full compilation verification
3. Verified `npm run build` on frontend (unchanged, still passes)
4. Confirmed only expected warning: `Paginated` unused (intentional — for future use)

---

## Remaining P1/P2 Items (Not P0 — Defer to Next Sprint)

| Item | Severity | Notes |
|------|----------|-------|
| Enforce ticket state transition matrix | High | Workflow gap |
| Centralize faculty conflict checks | High | Weekly/makeup/substitute |
| Wrap CSV imports in transactions | High | Atomic imports |
| Move session-status guard to repository layer | Medium | Cancelled sessions |
| Add DB transactions to multi-statement mutations | Medium | `create_ticket`, `update_ticket`, etc. |
| Extract `components.tsx` into feature folders | High | 8,851-line monolith |
| Extract `App.tsx` state into feature hooks | High | 61 useState hooks |
| Backend service layer extraction | High | `repositories.rs` = 5,903 lines |
| Add inline validation + dirty-state guards | Medium | Form UX |
| Add CSS design tokens | Medium | Visual consistency |
| Daily attendance rollup table | Medium | O(1) reporting |

---

*End of P0 Fix Report*
