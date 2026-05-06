# Regression Fix Report — Delta Review Follow-Up

**Date:** 2026-04-30  
**Team:** 5 specialist agents (3 in Phase 1 parallel, 2 in Phase 2 parallel)  
**Build Status:** ✅ Backend compiles (`cargo build` passes, 1 expected `Paginated` warning)

---

## Agent Roster & Assignments

| Phase | Agent | Fix | Files Modified | Status |
|-------|-------|-----|---------------|--------|
| 1 | **Regression Fixer** | R1: Phantom sessions from soft-deleted slots | `repositories.rs` | ✅ Done |
| 1 | **Stability Fixer** | R2: `refresh_escalations` DoS on every read | `routes/tickets.rs` | ✅ Done |
| 1 | **Security Fixer** | R3: CSV formula injection bypass | `routes/export.rs` | ✅ Done |
| 2 | **Security Fixer** | R4: 90 explicit `AppError::internal` leaks | `error.rs` | ✅ Done |
| 2 | **Workflow Fixer** | C6: Cancelled sessions spawn phantom attendance | `repositories.rs`, `routes/faculty.rs` | ✅ Done |

---

## Fix 1: Phantom Sessions from Soft-Deleted Timetable Slots (R1)

**Agent:** Regression Fixer  
**File:** `backend/src/repositories.rs`

**Bug:** Migration 41 added `deleted_at` to `timetable_slots`, but the CTEs in `list_faculty_today_sessions` and `list_all_today_sessions` never filtered it. Deleted slots continued generating lecture sessions every day.

**Fix:** Added `AND ts.deleted_at IS NULL` to 3 CTE queries:
- `list_faculty_today_sessions` merged-slot CTE
- `list_all_today_sessions` template upsert CTE
- `list_all_today_sessions` merged CTE

---

## Fix 2: `refresh_escalations` DoS on Every Read (R2)

**Agent:** Stability Fixer  
**File:** `backend/src/routes/tickets.rs`

**Bug:** `list_tickets` called `refresh_escalations` on every request, performing a full-table write storm (SELECT all tickets + UPDATE + INSERT history for each changed ticket).

**Fix:**
- **Removed** `refresh_escalations` from `list_tickets`
- **Added** `refresh_escalations` to `create_ticket` (after ticket creation)
- **Added** `refresh_escalations` to `update_ticket` (after ticket update)

Escalation status now refreshes only when data changes, not on every page load.

---

## Fix 3: CSV Formula Injection Bypass (R3)

**Agent:** Security Fixer  
**File:** `backend/src/routes/export.rs`

**Bug:** The `field()` function checked for formula triggers (`=`, `+`, `-`, `@`, `\t`) **after** CSV escaping. A payload like `=cmd|' /C calc'!A0,extra` was wrapped in quotes first (`"..."`), so the first character was `"` and no `'` prefix was added.

**Fix:** Check formula triggers on the **raw original string** before any escaping:
```rust
let needs_defuse = s.starts_with(|c| c == '=' || c == '+' || c == '-' || c == '@' || c == '\t');
// ... then escape ...
if needs_defuse {
    result.insert(0, '\'');
}
```

---

## Fix 4: 90 Explicit `AppError::internal` Leaks (R4)

**Agent:** Security Fixer  
**File:** `backend/src/error.rs`

**Bug:** Route handlers called `AppError::internal(format!(...))` directly, bypassing the `From<String>` sanitizer and exposing DB pool errors, filesystem paths, and SQLite internals.

**Fix:** Changed `AppError::internal` itself to log + sanitize:
```rust
pub fn internal(msg: impl Into<String>) -> Self {
    let m = msg.into();
    tracing::error!("Internal error: {}", m);
    Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: "An internal error occurred".to_string() }
}
```

This single change sanitizes **all 90 explicit calls** across every route file.

---

## Fix 5: Cancelled Sessions Spawn Phantom Attendance (C6)

**Agent:** Workflow Fixer  
**Files:** `backend/src/repositories.rs`, `backend/src/routes/faculty.rs`

**Bug:** GET `session_attendance` called `ensure_session_students` which inserted `Absent` rows for every student without checking if the session was cancelled.

**Fix — 3 layers of defense:**
1. **Repository `ensure_session_students`**: Returns `Ok(())` early if session status is `"Cancelled"`
2. **Repository `mark_attendance`**: Returns error `"Cannot mark attendance for a cancelled session"` if status is `"Cancelled"`
3. **Route `session_attendance` GET**: Returns empty list `Ok(Json(vec![]))` if session is cancelled, never calling the repository function

---

## Coordination Notes

### Phase 1 — 3 Agents in Parallel (No File Overlap)
- **Regression Fixer** → `repositories.rs` (CTEs for today-sessions)
- **Stability Fixer** → `routes/tickets.rs` (refresh_escalations calls)
- **Security Fixer** → `routes/export.rs` (CSV field function)

Zero file overlap → zero conflicts.

### Phase 2 — 2 Agents in Parallel (No File Overlap)
- **Security Fixer** → `error.rs` (AppError::internal)
- **Workflow Fixer** → `repositories.rs` + `routes/faculty.rs` (attendance guards)

Zero file overlap → zero conflicts.

### Manager Verification
1. Reviewed all agent reports for completeness
2. Ran `cargo build` for final compilation verification
3. Confirmed only expected warning: `Paginated` unused (intentional — for future use)

---

## Remaining Critical Issues (From Delta Review)

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| C1 | Ticket state machine (any → any transitions) | P1 | Still open |
| C3 | `get_student_timeline` loads unscoped global data | P1 | Still open |
| C4 | Rate limiter allows username enumeration | P1 | Still open |
| C5 | CSV imports without transactions | P1 | Still open |
| C7 | Viewer/agent global read scope | P2 | Still open |
| — | `components.tsx` monolith (8,851 lines) | P0 | Still open |
| — | `App.tsx` god-component (61 useState) | P0 | Still open |
| — | `repositories.rs` mixing SQL + business logic | P0 | Still open |

---

*End of Regression Fix Report*
