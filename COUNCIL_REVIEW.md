# SAATHI Critical Review — Council of 5

**Date:** 2026-04-30  
**Scope:** Full-stack review (Database · Workflow · UI/UX · Architecture · Security & Stability)  
**Reviewers:** 5 specialized agents analyzing `backend/src/db.rs`, `repositories.rs`, `routes/`, `frontend/src/components.tsx`, `App.tsx`, `api.ts`, deployment config

---

## Executive Summary

SAATHI is a functionally rich school-operations platform with 39 migrations, full CRUD, faculty attendance, bulk imports, and reporting. However, **rapid phased development (5 major phases) has accumulated significant structural and security debt**. The most dangerous issues cluster around:

1. **Security:** Hardcoded JWT fallback secret + default test accounts with weak passwords = immediate full-system compromise vector.
2. **Data Integrity:** Missing foreign keys, unbounded table growth, no transactions on multi-step operations, and cascading deletes that destroy historical attendance.
3. **Architecture:** A 312KB monolithic `components.tsx` and 2,300-line `App.tsx` god-component make parallel development nearly impossible.
4. **Workflow Gaps:** No enforced state machines, no calendar conflict checks on substitutions/makeups, and zero outbound notification delivery despite rich delivery metadata in the schema.

**Cross-cutting theme:** The codebase is excellent at *storing* data but weak at *guaranteeing* its integrity, *coordinating* operational workflows, and *scaling* its UI architecture.

---

## 1. Database Design Review

### 🔴 Critical Findings

| # | Finding | Severity | Evidence |
|---|---------|----------|----------|
| 1 | **Zero secondary indexes on `tickets`** — Full table scans on every list, filter, and escalation refresh | High | `repositories.rs:98-122`, `repositories.rs:124-226` |
| 2 | **Migration 39 regressed FKs on `lecture_sessions`** — `timetable_slot_id` and `actual_faculty_user_id` lost their `ON DELETE` constraints | High | `db.rs:857-882` |
| 3 | **`tickets.school_id` has no FK** — Orphan rows possible if school deleted | Medium | `db.rs:387-408` |
| 4 | **Unbounded growth without archival** — `attendance_records` grows ~200k rows/year/school; reporting queries scan full history | Medium | `repositories.rs:251-305` |
| 5 | **Denormalized `school_name` in `tickets` drifts** — School rename doesn't backfill historical tickets | Medium | `repositories.rs:318`, `db.rs:50-70` |
| 6 | **Missing `updated_at` on mutable tables** — `students`, `users`, `ticket_comments` lack audit timestamps | Low | Global |
| 7 | **`refresh_escalations` blocks writers** — N+1 updates in a write transaction under WAL | Low | `repositories.rs:124-226` |

### Immediate Actions

```sql
-- Add in migration 40 (or as standalone indexes)
CREATE INDEX idx_tickets_school_id ON tickets(school_id);
CREATE INDEX idx_tickets_updated_at ON tickets(updated_at);
CREATE INDEX idx_tickets_sla_due_at ON tickets(sla_due_at);
CREATE INDEX idx_tickets_status ON tickets(status);
CREATE INDEX idx_ticket_comments_ticket_id ON ticket_comments(ticket_id);
CREATE INDEX idx_ticket_history_ticket_id ON ticket_history(ticket_id);
CREATE INDEX idx_attendance_records_session_id ON attendance_records(lecture_session_id);
CREATE INDEX idx_attendance_records_student_id ON attendance_records(student_id);
CREATE INDEX idx_students_school_id ON students(school_id);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
```

### 🎯 Big Bet: Daily Attendance Rollup Table

Create a `daily_attendance_summary` materialized table pre-aggregating per school/grade/track/batch/date. Replace heavy subquery counts in `attendance_summary`, `chronic_absentees`, and `list_all_today_sessions` with O(1) lookups. Populate via `AFTER INSERT/UPDATE` trigger or nightly job.

---

## 2. Workflow & Business Logic Review

### 🔴 Critical Findings

| # | Finding | Severity | Evidence |
|---|---------|----------|----------|
| 1 | **No ticket state machine** — Any status can jump to any status (e.g., `Closed` → `Open` without gate) | High | `repositories.rs:368-448`, `routes/tickets.rs` |
| 2 | **Faculty calendar conflicts unenforced** — Substitution, makeup, and weekly override bypass double-booking checks | High | `repositories.rs:4209-4223`, `repositories.rs:3781-3800`, `repositories.rs:3519-3598` |
| 3 | **Timetable deletion cascades to historical attendance** — `ON DELETE CASCADE` from `timetable_slots` → `lecture_sessions` → `attendance_records` destroys history | High | `db.rs:671-678`, `repositories.rs:3410-3417` |
| 4 | **Zero outbound notifications** — Rich `delivery_status`/`channel` schema but no SMTP/SMS/WhatsApp gateway; SLA breaches are silent | High | `routes/tickets.rs`, `components.tsx:7385-7411` |
| 5 | **CSV imports row-by-row without transactions** — Mid-import failure leaves partial state | Medium | `routes/imports.rs` |
| 6 | **Attendance API accepts marks for cancelled sessions** — Frontend disables UI but API doesn't validate | Medium | `routes/faculty.rs:244-262`, `repositories.rs:4152-4207` |
| 7 | **Viewer/agent cross-school read by design** — `scope_filter` returns `None` for these roles | Medium | `auth.rs:93-102` |

### Recommended State Transition Matrix (Tickets)

| From → To | admin | aom | agent | faculty | viewer |
|-----------|-------|-----|-------|---------|--------|
| Open → In Progress | ✓ | ✓ | ✓ | — | — |
| Open → Pending | ✓ | ✓ | ✓ | — | — |
| In Progress → Resolved | ✓ | ✓ | ✓ | ✓ | — |
| Pending → Resolved | ✓ | ✓ | ✓ | — | — |
| Resolved → Closed | ✓ | ✓ | — | — | — |
| Any → Open (reopen) | ✓ | — | — | — | — |
| Closed → * | ✗ | ✗ | ✗ | ✗ | ✗ |

### 🎯 Big Bet: Event-Driven Operational Bus

Introduce an `operational_events` table (SQLite-backed queue) + async worker. Publish domain events (`TicketCreated`, `SlaBreached`, `SessionSubstituted`, `AttendanceMarked`). Consumers handle: notification dispatch (SMTP/Twilio/WhatsApp), SLA refresh, audit logging, approval workflows, and real-time UI (WebSocket/SSE). Transforms SAATHI from passive CRUD into an active coordination platform.

---

## 3. UI/UX Design Review

### 🔴 Critical Findings

| # | Finding | Severity | Evidence |
|---|---------|----------|----------|
| 1 | **Monolithic `components.tsx` (312KB, 8,851 lines)** — Merge conflicts, slow hot reload, no tree-shaking | High | `components.tsx` |
| 2 | **Accessibility gaps** — No focus traps in modals, no `aria-live` for alerts, color-only priority indicators, native `confirm()` for destructive actions | High | `components.tsx` (multiple modals), `TicketList` |
| 3 | **Mobile "More" menu unusable at scale** — 20+ flat buttons, no search/filter | High | `MobileMoreMenu` (~line 561) |
| 4 | **No inline form validation** — Users discover required fields only on submit | Medium | `CreateTicketModal`, `TicketEditForm`, `MasterDataPanel` |
| 5 | **Ticket reply draft partially lost** — Only `body` saved to `localStorage`; channel/audience/recipient lost | Medium | `App.tsx:~241-251` |
| 6 | **No dirty-state guard** — Closing modals silently discards edits | Medium | Global |
| 7 | **Ad-hoc design tokens** — Colors hardcoded ~200 places; faculty app uses different primary color | Medium | `styles.css` |

### Role-Based UX Pain Points

| Role | Pain Point | Quick Win |
|------|-----------|-----------|
| Admin | 25+ sidebar items; no favorites | Add "Pinned" section in localStorage |
| AOM | Faculty features mixed with admin settings | Collapse "Settings" by default on mobile |
| Faculty | Separate app; no ticket visibility | Add read-only "My School Tickets" tab |
| Viewer | Sees buttons that error or do nothing | Filter nav server-side by capability |

### 🎯 Big Bet: React Query + Optimistic Updates + Component Extraction

Migrate from `App.tsx` god-component (50+ `useState`, 30+ loaders) to `@tanstack/react-query` with feature-based folders. Optimistic mutations for resolve/assign/status changes feel instant. Background refetching eliminates manual refresh. Expected: 60% reduction in `App.tsx` LOC.

---

## 4. Code Architecture & Refactoring Review

### 🔴 Critical Findings

| # | Finding | Severity | Evidence |
|---|---------|----------|----------|
| 1 | **`App.tsx` god-component (2,298 lines)** — 90 `useState`, ~60 load/save handlers, 20+ admin panel branches | High | `App.tsx:166-256`, `App.tsx:402-1598` |
| 2 | **`repositories.rs` conflates SQL with business logic** — SLA calc, queue assignment, audit logging, validation all in one 5,903-line file | High | `repositories.rs:307-366` |
| 3 | **No pagination on list endpoints** — Full tables returned for tickets, schools, students, comments | High | `routes/tickets.rs:20`, `api.ts:114` |
| 4 | **String-dispatch API client (`api.ts`)** — Runtime errors on typos; zero compile-time safety | Medium | `api.ts:177` |
| 5 | **Massive DRY violations** — 1,295 table-element references, 20+ inline forms | Medium | `components.tsx` |
| 6 | **`models.rs` mixes infrastructure + domain + DTO** — `AppState`, `Claims`, entities, request structs all together | Medium | `models.rs` |
| 7 | **No tests, no ESLint, no Prettier** — Zero regression safety | High | `package.json` |

### Target Architecture (Frontend)

```
src/
  api/client.ts          # fetch wrapper
  api/tickets.ts         # typed API
  components/ui/         # Button, Input, DataTable, Modal
  components/layout/     # Sidebar, Topbar
  features/tickets/      # hooks.ts + components
  features/schools/
  features/faculty/
  contexts/              # AuthContext, AppErrorContext
```

### Target Architecture (Backend)

```
src/
  routes/          # thin: extract → service → respond
  services/        # business rules, validation, side effects
  repositories/    # pure SQL only
  models/
    domain.rs      # DB entities
    dto.rs         # request/response structs
    auth.rs        # Claims, CurrentUser
```

### 🎯 Big Bet: Feature-Based Folders + Typed Contracts

4-week migration to feature folders + Zod (frontend) / `garde` (backend) for runtime API contract validation. No heavy frameworks (no Redux, no tRPC server). Extract 10 UI primitives (`DataTable`, `FormStack`, `Modal`, `SearchInput`) collapsing ~3,000 lines.

---

## 5. Security & Stability Review

### 🔴 Critical Findings

| # | Finding | Severity | Evidence |
|---|---------|----------|----------|
| 1 | **Hardcoded fallback JWT secret** — If `JWT_SECRET` env var unset, predictable token enables admin forgery | **Critical** | `main.rs:46` |
| 2 | **Default & test users seeded in production** — `admin`/`admin123`, `aom1`/`aom123`, etc. created on every startup | **Critical** | `db.rs:466-471`, `db.rs:1052-1089` |
| 3 | **Horizontal privilege escalation (ticket create)** — `school_id: null` + `school_name: "Victim School"` bypasses `enforce_school_scope` | High | `routes/tickets.rs:28-33`, `routes/tickets.rs:41-50` |
| 4 | **No rate limiting on login** — Unlimited brute-force speed against weak test accounts | High | `routes/auth.rs:11-31` |
| 5 | **CSV formula injection in exports** — `=cmd|' /C calc'!A0` in ticket descriptions executes on Excel open | High | `routes/export.rs:6-12` |
| 6 | **Unbounded list queries (DoS/OOM)** — No LIMIT on tickets, comments, students; 256MB Fly.io VM will OOM-kill | Medium | `repositories.rs:98-122` |
| 7 | **DB restore accepts unbounded files** — No size limit; can exhaust disk/RAM | Medium | `routes/admin.rs:211-276` |
| 8 | **No database transactions** — `update_ticket` does 10+ separate SQLs; partial failure corrupts audit | Medium | `repositories.rs` (global) |

### Additional Observations

| Issue | Location | Note |
|-------|----------|------|
| Overly permissive CORS | `main.rs:58-61` | `allow_origin(Any)` — restrict to Vercel frontend |
| Error message leakage | `error.rs:39-42` | Full SQL details exposed in 500 responses |
| Missing auth on admin read endpoints | `admin.rs:96-152` | SLA policies, assignment rules readable by any authenticated user |
| JWT in `sessionStorage` | `frontend/src/api.ts:20` | Vulnerable to XSS if unsafe DOM APIs ever introduced |

### 🎯 Big Bet: OAuth2/OIDC + Security CI Pipeline

Migrate from password-based auth to **OAuth 2.0 / OIDC** (Google Workspace / Microsoft Entra ID). Eliminates weak passwords, brute-force, and session management complexity. Add `cargo-audit`, SQL injection fuzzer, and `zaproxy` baseline scan to CI.

---

## Synthesized Priority Matrix

### Must Fix Before Next Deployment (P0)

| # | Issue | Domain | Owner |
|---|-------|--------|-------|
| 1 | Remove hardcoded JWT fallback; abort startup if `JWT_SECRET` missing | Security | Backend |
| 2 | Gate test user seeding behind `TEST_SEED=1` env var; force admin password reset | Security | Backend |
| 3 | Resolve school BEFORE enforcing scope in ticket create/update | Security | Backend |
| 4 | Add rate limiting to login endpoint | Security | Backend |
| 5 | Prefix formula chars in CSV export fields | Security | Backend |

### High Priority (P1) — Next 2-4 Weeks

| # | Issue | Domain | Owner |
|---|-------|--------|-------|
| 6 | Add secondary indexes (tickets, comments, attendance, audit) | Database | Backend |
| 7 | Restore FKs on `lecture_sessions` (migration 40) | Database | Backend |
| 8 | Enforce ticket status transition matrix + role gates | Workflow | Backend |
| 9 | Add calendar conflict checks to substitute + makeup + weekly override | Workflow | Backend |
| 10 | Protect timetable deletion from cascading attendance loss | Workflow | Backend |
| 11 | Add pagination (limit/offset) to all list endpoints | Stability | Backend |
| 12 | Add database transactions to multi-statement mutations | Stability | Backend |
| 13 | Split `components.tsx` into domain folders | Architecture | Frontend |
| 14 | Extract feature hooks from `App.tsx` (god-component split) | Architecture | Frontend |
| 15 | Add focus traps + `aria-live` regions | UI/UX | Frontend |

### Medium Priority (P2) — Next 1-3 Months

| # | Issue | Domain | Owner |
|---|-------|--------|-------|
| 16 | Introduce service layer; separate business logic from repositories | Architecture | Backend |
| 17 | Replace string-dispatch API with typed HTTP client | Architecture | Frontend |
| 18 | Extract reusable DataTable, FormStack, Modal primitives | UI/UX | Frontend |
| 19 | Add inline form validation + dirty-state guards | UI/UX | Frontend |
| 20 | Add CSS design tokens (custom properties) | UI/UX | Frontend |
| 21 | Add `updated_at` timestamps to mutable tables | Database | Backend |
| 22 | Create daily attendance rollup table | Database | Backend |
| 23 | Add offline indicator + connectivity hook | UI/UX | Frontend |
| 24 | Restrict CORS to known frontend origin | Security | Backend |

### Strategic Bets (P3) — 3-6 Months

| # | Initiative | Domains | Impact |
|---|------------|---------|--------|
| A | **Event-Driven Operational Bus** — `operational_events` table + async worker for notifications, SLA, approvals | Workflow + Security + Stability | Transforms CRUD into active platform |
| B | **React Query + Optimistic Updates** — Server state management, background refetch, instant mutations | UI/UX + Architecture | 60% App.tsx reduction, instant UX |
| C | **OAuth2/OIDC Migration** — Google Workspace / Entra ID integration | Security | Eliminates password risk entirely |
| D | **Feature-Based Folders + Typed Contracts** — Zod/garde validation, design system extraction | Architecture + UI/UX | Enables parallel development at 2× scale |
| E | **Daily Attendance Rollup** — Materialized summary table with trigger/nightly population | Database + Stability | O(1) reporting lookups |

---

## Conclusion

The council's consensus: **SAATHI's functional breadth is impressive, but its production readiness is compromised by security gaps and structural debt accumulated across 5 rapid development phases.**

The three most urgent actions are all security-related (JWT fallback, test accounts, scope bypass). These can be fixed in a single day and should block any production deployment.

The highest-leverage structural investment is **splitting the frontend monolith** (`components.tsx` + `App.tsx`) into feature-based folders. This single change unlocks parallel development, reduces the blank-screen regression pattern seen in recent commits, and enables adoption of React Query and a design system.

On the backend, the combination of **service layer extraction**, **database transactions**, and **pagination** will stabilize the system as data volume grows from demo-scale to real school-scale (1000+ students, 10k+ tickets).

The "big bets" (Event Bus, React Query, OAuth2) are not luxuries — they are the architectural foundations needed for SAATHI to operate as a *coordination platform* rather than a *data entry system*. School operations require real-time awareness, automated escalation, and reliable communication. The current codebase stores all the right data but does not yet *act* on it.

---

*Report generated by Council of 5 specialized review agents. Each finding includes specific file/line references in the agent sub-reports (see task outputs).*
