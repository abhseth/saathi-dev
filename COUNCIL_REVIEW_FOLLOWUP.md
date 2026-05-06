# SAATHI Critical Review — Follow-Up (Post-Security-Fixes)

**Date:** 2026-04-30  
**Scope:** Full-stack re-review after 5 P0 security fixes deployed  
**Status:** 5 P0 fixes verified ✅ · 20+ remaining issues identified  

---

## Executive Summary

The 5 critical security fixes are **solid and verified**. However, **zero structural improvements** have been made to the underlying architecture, workflows, or UI since the first review. The codebase remains in the same state of rapid-development debt:

- **Database:** Migration 39 actually **worsened** referential integrity by stripping FKs from `lecture_sessions`. Needs urgent Migration 40.
- **Workflow:** 6 of 7 gaps remain — the most dangerous being **timetable deletion cascading to destroy all historical attendance forever**.
- **UI/UX:** `components.tsx` is still 8,851 lines. `App.tsx` still has 61 `useState` hooks. No pagination, no accessibility fixes.
- **Architecture:** `repositories.rs` still mixes SQL with business rules. No service layer. No pagination. No tests.
- **Security:** P0 fixes verified, but 5 significant remaining issues exist (unbounded queries, error leakage, CORS, missing transactions, admin endpoint exposure).

**Key insight:** The security fixes were tactical band-aids. The strategic debt — monolithic files, missing state machines, no data integrity guards, zero test coverage — is unchanged and growing more expensive to fix with each new feature.

---

## 1. Database Design — Follow-Up

### P0: Migration 40 Required — FKs Stripped in Migration 39

**Trend:** 🔴 **WORSE** than last review.

Migration 29 created `lecture_sessions` with FKs. **Migration 39 recreates it with zero FK clauses** for `timetable_slot_id`, `actual_faculty_user_id`, `subject_id`, and `school_id`. This silently broke cascading deletion and referential integrity.

**Action:** Add Migration 40 to `db.rs` that:
1. Cleans orphaned rows that would violate restored FKs
2. Recreates `lecture_sessions` with all FK constraints
3. Rebuilds indexes on `date`, `faculty`, `school`, `subject`, `slot`

### P1: Missing Indexes — Immediate Zero-Downtime Fix

Run this directly against any SAATHI DB:

```sql
CREATE INDEX IF NOT EXISTS idx_tickets_school      ON tickets(school_id);
CREATE INDEX IF NOT EXISTS idx_tickets_updated     ON tickets(updated_at);
CREATE INDEX IF NOT EXISTS idx_tickets_escalation  ON tickets(escalation_status, sla_due_at);
CREATE INDEX IF NOT EXISTS idx_tickets_status      ON tickets(status);
CREATE INDEX IF NOT EXISTS idx_attendance_student  ON attendance_records(student_id);
CREATE INDEX IF NOT EXISTS idx_students_school     ON students(school_id);
CREATE INDEX IF NOT EXISTS idx_students_batch      ON students(school_id, grade_level, track, batch_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_created   ON audit_log(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity    ON audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_ticket_history_ticket ON ticket_history(ticket_id);
CREATE INDEX IF NOT EXISTS idx_ticket_comments_ticket ON ticket_comments(ticket_id);
CREATE INDEX IF NOT EXISTS idx_users_role          ON users(role);
CREATE INDEX IF NOT EXISTS idx_faculty_assignments_faculty ON faculty_assignments(faculty_user_id);
CREATE INDEX IF NOT EXISTS idx_faculty_assignments_school  ON faculty_assignments(school_id);
```

### P1: `refresh_escalations` N+1 Writes Under WAL

**Current:** Loops through all tickets, issuing one UPDATE per changed ticket + history inserts. Blocks all other writers.

**Fix:** Convert to a single set-based UPDATE using a CTE, then batch-insert history rows.

### P2: Missing `updated_at` on Mutable Tables

```sql
ALTER TABLE students ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'));
ALTER TABLE users ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'));
ALTER TABLE ticket_comments ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'));
```

### 🎯 Big Bet: Daily Attendance Rollup (Trigger-Based)

Create `daily_attendance_rollup` maintained by `AFTER INSERT/UPDATE/DELETE` triggers on `attendance_records`. Eliminates the 3 correlated `COUNT(*)` subqueries in `list_faculty_today_sessions` and makes Phase 4 reports instantaneous. Full schema + triggers + backfill SQL available in the Database agent's full report.

---

## 2. Workflow — Follow-Up

### P0: Timetable Deletion Destroys Historical Attendance

**Trend:** 🔴 **UNCHANGED — Most dangerous remaining gap.**

`ON DELETE CASCADE` from `timetable_slots` → `lecture_sessions` → `attendance_records` permanently erases every past session instance and every student attendance mark when a slot is deleted.

**Fix:** Replace hard `DELETE` with **soft-delete** (`deleted_at` timestamp) + guard that prevents deletion if historical sessions exist.

### P1: No Ticket State Machine

**Trend:** 🟡 **UNCHANGED.**

Any status can jump to any status. `Open → Closed` without resolution, `Closed → Resolved` backwards.

**Fix:** Add a hardcoded transition map in `update_ticket`:
```rust
let allowed = match before_status {
    "Open" => &["In Progress", "Pending", "Resolved", "Closed"],
    "In Progress" => &["Open", "Pending", "Resolved", "Closed"],
    "Pending" => &["Open", "In Progress", "Resolved", "Closed"],
    "Resolved" => &["Open", "In Progress", "Pending", "Closed"],
    "Closed" => &["Open"],  // reopen only
    _ => &[],
};
```

### P1: CSV Imports Without Transactions

**Trend:** 🟡 **UNCHANGED.**

Row-by-row imports with no `BEGIN TRANSACTION` → partial state on failure.

**Fix:** Wrap each import in `conn.execute("BEGIN")` ... `COMMIT`/`ROLLBACK`.

### P1: Calendar Conflicts Unenforced

**Trend:** 🟡 **UNCHANGED.**

Weekly overrides, makeup sessions, and substitutions bypass double-booking checks that exist only in the recurring `timetable_slots` path.

**Fix:** Extract a centralized `assert_faculty_available()` helper called from all 4 write paths.

### P2: Cancelled Sessions Spawn Phantom Attendance

**Trend:** 🟡 **PARTIALLY FIXED (route-level only).**

The POST `mark_attendance` route rejects cancelled sessions, but the GET `session_attendance` route still calls `ensure_session_students` which inserts `Absent` rows without checking status.

**Fix:** Move status guard into the repository layer (`ensure_session_students`, `mark_attendance`).

### 🎯 Big Bet: Event-Driven Operational Bus

SQLite-backed `outbox` table (`operation_events`) + background Tokio worker. Decouples notifications, SLA refresh, audit logging from request handlers. Enables SMTP/SMS retries, real-time SSE updates, and approval workflows without Redis/RabbitMQ.

---

## 3. UI/UX — Follow-Up

### P0: `components.tsx` Still 8,851 Lines

**Trend:** 🔴 **ZERO PROGRESS.**

Every panel, modal, table, form lives in one file. Blocks code review, guarantees merge conflicts, prevents tree-shaking.

### P1: Accessibility Gaps Persist

**Trend:** 🔴 **UNCHANGED.**

- Color-only priority indicators (WCAG 1.4.1 violation)
- 4 native `confirm()` calls + 5 `alert()` calls
- No focus traps on modals
- Zero `aria-live` regions

### P1: Zero Pagination + Horizontally Scrolling Tables

**Trend:** 🟡 **UNCHANGED.**

Every list renders full array via `.map()`. 10 tables force two-axis scrolling on mobile. No sticky headers.

### P1: Role-Based IA Leaks Destructive Buttons

**Trend:** 🟡 **UNCHANGED.**

Viewer sees "Assign to me" and "Resolve" buttons. Admin sidebar has ~25 flat items. Mobile "More" menu is a 20+ item text list.

### P2: No Dirty-State Guards / Inline Validation

**Trend:** 🟡 **UNCHANGED.**

Only password field has inline validation. No `beforeunload` guards. No disabled submit states.

### 🎯 Big Bet: React Query + Feature Extraction

Migrate from 61 `useState` hooks in `App.tsx` to `@tanstack/react-query`. Extract features into folders. Expected: `App.tsx` shrinks from 2,298 → ~300 lines, `components.tsx` → 0 lines. Automatic caching, deduplication, background refresh, optimistic updates.

---

## 4. Architecture — Follow-Up

### P0: `repositories.rs` Still 5,903 Lines

**Trend:** 🔴 **UNCHANGED.**

SQL, SLA calculation, queue assignment, audit logging, validation, email regex checks all in one file.

### P1: No Pagination on List Endpoints

**Trend:** 🟡 **UNCHANGED.**

Full table scans over the wire. Will OOM 256MB Fly VM at scale.

### P1: String-Dispatch API (`api.ts`)

**Trend:** 🟡 **UNCHANGED.**

Runtime errors on typos. Zero compile-time safety.

### P1: `models.rs` Mixes Infrastructure + Domain + DTO

**Trend:** 🟡 **UNCHANGED.**

`AppState`, `Claims`, entities, request structs all together.

### P2: Missing Tooling

**Trend:** 🟡 **UNCHANGED.**

No ESLint, no Prettier, no tests, no React Query/SWR, no `cargo clippy`.

### 🎯 Big Bet: Feature-Based Folders + Typed Contracts

4-week migration:
1. Frontend: `src/features/{tickets,schools,faculty,admin}/` with colocated hooks + components
2. Backend: `services/` layer for business rules, `repositories/` for pure SQL
3. Contracts: Zod (frontend) + `garde` (backend) for runtime API validation

---

## 5. Security & Stability — Follow-Up

### P0 Fix Verification

| Fix | Status | Gaps |
|-----|--------|------|
| JWT fallback removed | ✅ Verified | `admin123` still hardcoded in Migration 20 |
| Test seeding gated | ✅ Verified | `TEST_SEED=1` required |
| Scope bypass fixed | ✅ Verified | No gaps found |
| Rate limiting | ✅ Verified | In-memory only — resets on deploy; unbounded HashMap growth possible |
| CSV injection | ✅ Verified | Only first char checked; pipe `\|` not sanitized |

### Top 5 Remaining Security Issues

| # | Finding | Severity | File |
|---|---------|----------|------|
| 1 | **Unbounded list queries** → DoS/OOM | High | `repositories.rs` (all `list_*` functions) |
| 2 | **Error message leakage** — SQL schema in 500 responses | Medium-High | `error.rs:26-28` |
| 3 | **Admin read endpoints without role checks** | Medium | `routes/admin.rs:96-152` |
| 4 | **JWT in `sessionStorage` + CORS `Any` + No CSP** | Medium-High | `api.ts:19-24`, `main.rs:58-61` |
| 5 | **No DB transactions** on multi-statement mutations | Medium | `repositories.rs` globally |

### Remediation Quick-Wins

1. **Pagination:** Add `LIMIT 1000` to all list queries; accept `?limit=`/`?offset=` in routes.
2. **Sanitize 500s:** `tracing::error!()` the real message, return generic `"An internal error occurred"`.
3. **Lock admin reads:** Add `require_admin_or_aom(&claims)?` to SLA policies, assignment rules, escalation policy, communication templates.
4. **Harden CORS:** Restrict to known Vercel origin; add CSP headers.
5. **Transactions:** Wrap `create_ticket`, `update_ticket`, `add_comment`, `mark_attendance`, `create_user` in `conn.transaction()`.

### 🎯 Big Bet: OAuth2/OIDC + Security CI Pipeline

Replace password auth with Google Workspace / Microsoft Entra ID. Add `cargo-audit`, `cargo-clippy`, grep checks for `allow_origin(Any)` and `sessionStorage.setItem("td:token"`, and `zaproxy` baseline scan in CI.

---

## Synthesized Priority Matrix (Post-Security-Fixes)

### P0 — Fix This Week

| # | Issue | Domain | Effort |
|---|-------|--------|--------|
| 1 | **Migration 40** — Restore FKs on `lecture_sessions` | Database | 2 hours |
| 2 | **Add secondary indexes** (tickets, attendance, audit, students) | Database | 30 min |
| 3 | **Timetable soft-delete** — Prevent CASCADE destruction of historical attendance | Workflow | 1 day |
| 4 | **Add pagination / LIMIT** to all list endpoints | Stability | 1 day |

### P1 — Fix This Month

| # | Issue | Domain | Effort |
|---|-------|--------|--------|
| 5 | **Enforce ticket state transition matrix** | Workflow | 1 day |
| 6 | **Centralize faculty conflict checks** (weekly, makeup, substitute) | Workflow | 2 days |
| 7 | **Wrap CSV imports in transactions** | Workflow | 1 day |
| 8 | **Move session-status guard into repository layer** | Workflow | 1 day |
| 9 | **Sanitize 500 error messages** | Security | 2 hours |
| 10 | **Lock down admin read endpoints** | Security | 2 hours |
| 11 | **Restrict CORS + add CSP headers** | Security | 2 hours |
| 12 | **Add DB transactions to multi-statement mutations** | Stability | 2 days |
| 13 | **Extract `components.tsx` into feature folders** | Architecture | 3-5 days |
| 14 | **Extract `App.tsx` state into feature hooks** | Architecture | 1 week |

### P2 — Fix Next Quarter

| # | Issue | Domain | Effort |
|---|-------|--------|--------|
| 15 | **Backend service layer extraction** | Architecture | 2 weeks |
| 16 | **Replace string-dispatch API with typed client** | Architecture | 3 days |
| 17 | **Add ESLint + Prettier + React Query** | Architecture | 2 days |
| 18 | **Add inline validation + dirty-state guards** | UI/UX | 2 days |
| 19 | **Add CSS design tokens** | UI/UX | 1 day |
| 20 | **Add pagination controls to frontend tables** | UI/UX | 2 days |
| 21 | **Add `updated_at` to mutable tables** | Database | 2 hours |
| 22 | **Daily attendance rollup table** | Database | 3 days |

### P3 — Strategic Bets (3-6 Months)

| Bet | Domains | Impact |
|-----|---------|--------|
| **Event-Driven Operational Bus** | Workflow + Security + Stability | Transforms CRUD into active platform |
| **React Query + Optimistic Updates** | UI/UX + Architecture | 60% App.tsx reduction, instant UX |
| **OAuth2/OIDC Migration** | Security | Eliminates password risk entirely |
| **Feature-Based Folders + Typed Contracts** | Architecture + UI/UX | Enables parallel development at 2x scale |
| **Daily Attendance Rollup** | Database + Stability | O(1) reporting lookups |

---

## Conclusion

The 5 P0 security fixes were **necessary and well-executed**, but they were tactical. The strategic debt has not diminished — in fact, Migration 39 **regressed** database integrity by stripping FKs.

**The most urgent actions this week:**
1. Deploy Migration 40 (restore FKs)
2. Add the index creation script
3. Replace timetable hard-delete with soft-delete
4. Add LIMIT to list endpoints

These 4 items can be done in **3-4 days** and will prevent data loss, OOM crashes, and integrity violations.

**The highest-leverage investment this quarter:** Splitting `components.tsx` + `App.tsx` into feature folders. This single change unlocks parallel development, reduces the blank-screen regression pattern, and enables adoption of React Query and a design system.

**The transformation from data-entry system to operational coordination platform** requires the "big bet" investments: Event Bus for notifications, React Query for UX, and OAuth2 for security. These are not luxuries — they are the foundations for scaling SAATHI from a demo to a real school-operations platform.

---

*Follow-up report generated by Council of 5 specialized review agents after P0 security fix deployment.*
