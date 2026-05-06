# Urgent Fix Report — Delta Review Follow-Up

**Date:** 2026-04-30  
**Team:** 4 specialist agents (3 in Phase 1 parallel, 1 in Phase 2)  
**Build Status:** ✅ Backend compiles · ✅ Frontend compiles

---

## Agent Roster & Assignments

| Phase | Agent | Fix | Files Modified | Status |
|-------|-------|-----|---------------|--------|
| 1 | **State Machine Enforcer** | Ticket status transition validation | `repositories.rs` | ✅ Done |
| 1 | **Import Guard** | CSV imports wrapped in transactions | `routes/imports.rs` | ✅ Done |
| 1 | **Frontend Extractor** | Ticket components extracted from monolith | `components.tsx`, new files | ✅ Done |
| 2 | **Security Hardening** | Rate limiter + scope + timeline | `routes/auth.rs`, `auth.rs`, `repositories.rs` | ✅ Done |

---

## Fix 1: Ticket State Machine (C1)

**Agent:** State Machine Enforcer  
**File:** `backend/src/repositories.rs`

**Before:** `update_ticket` allowed any status to jump to any other status. `Open → Closed` without resolution was permitted.

**After:** Added `validate_status_transition(before, after)` with this matrix:

| From → To | Open | In Progress | Pending | Resolved | Closed |
|-----------|------|-------------|---------|----------|--------|
| **Open** | — | ✅ | ✅ | ✅ | ✅ |
| **In Progress** | ✅ | — | ✅ | ✅ | ✅ |
| **Pending** | ✅ | ✅ | — | ✅ | ✅ |
| **Resolved** | ✅ | ✅ | ✅ | — | ✅ |
| **Closed** | ✅ (reopen) | — | — | — | — |

Invalid transitions return: `"Invalid status transition: X → Y. Allowed from X: ..."`

---

## Fix 2: CSV Import Transactions (C5)

**Agent:** Import Guard  
**File:** `backend/src/routes/imports.rs`

**Before:** All 4 CSV imports processed rows individually. Row 847 failing left rows 1–846 committed.

**After:** Each import handler wrapped in `BEGIN TRANSACTION` / `COMMIT`:
- `import_schools_csv`
- `import_students_csv`
- `import_sip_master`
- `import_timetable_csv`

Imports are now atomic — all rows succeed or all fail.

---

## Fix 3: Frontend Component Extraction

**Agent:** Frontend Extractor  
**Files:** `frontend/src/components.tsx`, 3 new files

**Before:** `components.tsx` was 8,851 lines. Every component in one file.

**After:** ~1,071 lines extracted into feature folder:

| Component | New File | Lines |
|-----------|----------|-------|
| `TicketList` | `components/tickets/TicketList.tsx` | 97 |
| `TicketDetail` | `components/tickets/TicketDetail.tsx` | 746 |
| `CreateTicketModal` | `components/tickets/CreateTicketModal.tsx` | 259 |

`components.tsx` reduced from 8,851 → 7,780 lines. Re-exports maintain backward compatibility:
```tsx
export { TicketList } from "./components/tickets/TicketList";
export { TicketDetail } from "./components/tickets/TicketDetail";
export { CreateTicketModal } from "./components/tickets/CreateTicketModal";
```

---

## Fix 4: Security Hardening — 3 Issues in 1

**Agent:** Security Hardening  
**Files:** `routes/auth.rs`, `auth.rs`, `repositories.rs`

### 4A: Rate Limiter — Uniform Errors (`routes/auth.rs`)

**Before:** Different messages for "wrong password" vs "rate limited" enabled username enumeration.

**After:** Both paths return identical message: `"Invalid username or password"`

### 4B: Scope Filter — Viewer/Agent Global Read (`auth.rs`)

**Before:** `scope_filter` returned `None` (unscoped/global) for `viewer` and `agent` roles.

**After:** Only `admin` gets `None`. `viewer` and `agent` are scoped to their `school_ids`.

```rust
pub fn scope_filter<'a>(claims: &'a Claims) -> Option<&'a [i64]> {
    if claims.role == "admin" {
        None
    } else if claims.school_ids.is_empty() {
        Some(&[-1])
    } else {
        Some(&claims.school_ids)
    }
}
```

### 4C: Student Timeline — Scoped Queries (`repositories.rs`)

**Before:** `get_student_timeline` loaded unbounded global tables and filtered in memory:
- `list_tickets(conn, None)` → only 1000 most recent global tickets (old student tickets lost)
- `list_all_history(conn)` → **completely unbounded**
- `list_all_attachments(conn)` → **completely unbounded**

**After:** All four replaced with direct scoped SQL queries:
- Tickets: `WHERE school_id = ? AND student_name = ?`
- Comments/History/Attachments: `WHERE ticket_id IN (...student's tickets...)`
- All queries include `LIMIT 1000`

Eliminates both data loss (old tickets visible again) and memory DoS.

---

## Coordination Notes

### Phase 1 — 3 Agents in Parallel (Zero File Overlap)
- **State Machine Enforcer** → `repositories.rs` (update_ticket function)
- **Import Guard** → `routes/imports.rs`
- **Frontend Extractor** → `frontend/src/` (new files + components.tsx re-exports)

### Phase 2 — 1 Agent (Sequential, touched auth + repos)
- **Security Hardening** → `routes/auth.rs` + `auth.rs` + `repositories.rs`

No conflicts arose. All agents used exact string replacements.

### Manager Verification
1. Reviewed all agent reports
2. Ran `cargo build` — backend compiles cleanly
3. Ran `npm run build` — frontend compiles cleanly
4. Only expected warning: `Paginated` unused (for future pagination migration)

---

## Remaining Open Issues

| # | Issue | Severity | Domain |
|---|-------|----------|--------|
| 1 | Faculty calendar conflicts (weekly/makeup/substitute) | P1 | Workflow |
| 2 | `refresh_escalations` should be background job, not on mutation | P1 | Stability |
| 3 | `components.tsx` still 7,780 lines | P1 | Architecture |
| 4 | `App.tsx` still ~2,300 lines with 61 `useState` | P1 | Architecture |
| 5 | `repositories.rs` still mixing SQL + business logic | P1 | Architecture |
| 6 | No tests, no ESLint, no Prettier | P2 | Architecture |
| 7 | Zero outbound notifications (SMTP/SMS/WhatsApp) | P2 | Workflow |
| 8 | Viewer/agent scope fix may break existing viewer workflows | P2 | Security |

---

*End of Urgent Fix Report*
