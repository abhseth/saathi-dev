# SAATHI Multi-Level Review Committee Report

**Date:** 2026-05-01  
**Committee:** 8 Expert Reviewers  
**Status:** All 8 Complete  
**Overall Verdict:** 🟡 **Conditional Deploy** — Address P0 items before production

---

## Committee Members

| # | Expertise | Status | Key Focus |
|---|-----------|--------|-----------|
| 1 | Security & Data Protection | ✅ Complete | Auth, RBAC, SQL injection, XSS, CSRF |
| 2 | UX/Usability & Accessibility | ✅ Complete | Mobile UX, a11y, IA, visual design |
| 3 | Architecture & Code Quality | ✅ Complete | Module organization, DRY, test coverage |
| 4 | Performance & Scalability | ✅ Complete | Query performance, bundle size, concurrency |
| 5 | Database & Data Integrity | ✅ Complete | Schema design, migrations, query correctness |
| 6 | Mobile & Frontend Engineering | ✅ Complete | React patterns, TypeScript, state management |
| 7 | Backend & API Design | ✅ Complete | REST conventions, validation, error handling |
| 8 | Product & Feature Strategy | ✅ Complete | Feature completeness, user workflows, roadmap |

---

## Cross-Cutting Critical Issues (Multiple Committee Members Agree)

### 🔴 C1: SQL Injection via String Interpolation
**Found by:** Security, Architecture, Database, Backend API  
**Location:** `backend/src/analytics.rs`, `backend/src/alerts.rs`, `backend/src/repositories.rs`, `backend/src/routes/automation.rs`  
**Details:** Dozens of queries use `format!("... IN ({list})")` or `format!("... WHERE x = {}", value)`. While `i64` arrays are low-risk, `week_a`/`week_b` in `analytics.rs::week_diff` are raw string inputs from API. `automation.rs::bulk_alert_action` interpolates `hours` into SQL.  
**Fix:** Use `rusqlite::params_from_iter` with `?` placeholders exclusively.

### 🔴 C2: N+1 Query Patterns
**Found by:** Performance, Architecture, Database  
**Location:** `backend/src/analytics.rs`, `backend/src/substitution_engine.rs`, `backend/src/repositories.rs`  
**Details:**
- `faculty_utilization_trend`: 100 faculty × 4 weeks = **400 queries**
- `rank_substitute_candidates`: 50 candidates × 3 lookups = **150 queries**
- `list_users`: N users + 1 = **N+1 queries**
- `health_trends`: 8 weeks × 3 queries = **24 queries**

**Fix:** Rewrite as bulk queries with `GROUP BY` and `IN` clauses.

### 🔴 C3: `App.tsx` Monolith — 3,043 Lines, 60+ useState Hooks
**Found by:** Architecture, Performance, UX, Frontend Engineering  
**Location:** `frontend/src/App.tsx`  
**Details:** All global state lives in one component. Any state change triggers re-render across entire tree. No `React.memo` on panels. 50+ components eagerly imported → 495KB bundle. No error boundaries.  
**Fix:** Extract state to context/Zustand, implement code splitting, add `React.memo` to panels, add error boundaries.

### 🔴 C4: Aggregation Cartesian Product in Analytics
**Found by:** Database, Performance  
**Location:** `backend/src/analytics.rs::faculty_stability`  
**Details:** Three independent `LEFT JOIN lecture_sessions` on `timetable_slot_id` without date correlation. A slot with 5 sub + 3 cancel + 92 actual = 1,380 rows.  
**Fix:** Use correlated subqueries or `COUNT(DISTINCT)`.

### 🔴 C5: Attendance Aggregation Inflation
**Found by:** Database  
**Location:** `backend/src/repositories.rs::attendance_summary`, `subject_attendance`  
**Details:** Groups by school/grade/track but not by session. Multiple periods per day multiply counts.  
**Fix:** Group by `ls.id` or use `COUNT(DISTINCT ar.id)`.

### 🔴 C6: No Error Boundaries + Silent API Failures
**Found by:** Frontend Engineering, UX  
**Location:** `frontend/src/App.tsx`, `frontend/src/components/faculty/FacultyApp.tsx`, many components  
**Details:** Not a single Error Boundary in the codebase. Any runtime exception white-screens the app. `catch { /* silently fail */ }` pattern found in 20+ locations. No retry logic, no request cancellation.  
**Fix:** Add ErrorBoundaries, replace silent catches with error state, add AbortController.

---

## By-Domain Findings

### 1. Security & Data Protection 🔒

| Rating | Finding |
|--------|---------|
| 🔴 | **JWT in sessionStorage** — XSS-vulnerable; should be `httpOnly` cookies |
| 🔴 | **No token refresh** — tokens don't expire; stolen token is valid forever |
| 🔴 | **Missing authorization on Phase 6 routes** — `create_notification`, `alert_inbox`, `mark_attendance_quick`, `list_announcements` lack proper scope checks |
| 🔴 | **SQL injection in `automation.rs::bulk_alert_action`** — `hours` interpolated into SQL |
| 🔴 | **`update_subject` ignores Path parameter** — uses body `id` instead of path `id`; authorization bypass risk |
| 🟡 | `format!("... IN ({list})")` string interpolation across repositories |
| 🟡 | No upload size limits on multipart endpoints |
| 🟡 | Weak password policy (min length not enforced) |
| 🟡 | No rate limiting on state-changing endpoints |
| 🟢 | CORS restricted to configurable origin |
| 🟢 | bcrypt password hashing |
| 🟢 | `require_auth` middleware on all protected routes |

**P0 Actions:**
1. Fix SQL injection in `automation.rs`
2. Fix `update_subject` to use `Path(id)`
3. Add role check to `POST /notifications`
4. Add `require_admin_or_aom` to `create_notification`
5. Enforce caller ownership on `alert_inbox`
6. Implement token expiry + refresh flow

---

### 2. UX/Usability & Accessibility 🎨

| Rating | Finding |
|--------|---------|
| 🔴 | **60+ views in `AdminView` union** — information architecture sprawl, discovery failure |
| 🔴 | **Modal focus management gaps** — many modals lack `role="dialog"`, `useModalFocus` not used consistently |
| 🔴 | **Touch targets below 44px** — `.day-nav-btn` (36px), `.sub-dot` (10px), `.alert-dismiss` (~20px) |
| 🔴 | **Color-only status communication** — WCAG 1.4.1 failure in heatmaps, health dashboard |
| 🔴 | **Emoji as accessible icons without hiding** — screen readers announce "alarm clock" instead of meaning |
| 🟡 | Wide tables on mobile compete with browser back-swipe |
| 🟡 | Inconsistent primary action colors (teal vs indigo) |
| 🟡 | Sparse form validation feedback |
| 🟡 | Crude loading states ("Loading…" text only, no skeletons) |
| 🟡 | Silent API failures — empty state indistinguishable from error |
| 🟡 | Mobile "More" sheet has 40+ items, no search or recents |
| 🟢 | Global `Escape` closes modals, `j`/`k` vim navigation |
| 🟢 | `useModalFocus` correctly traps Tab cycles (where used) |
| 🟢 | Offline awareness with `OfflineBanner` |
| 🟢 | Reply draft persistence in `localStorage` |
| 🟢 | Safe-area handling for iPhone dynamic island |
| 🟢 | Print styles optimized for timetables |

**P0 Actions:**
1. Add `role="dialog"` + `aria-modal="true"` to all modals
2. Enlarge touch targets to 44px minimum
3. Add text labels to all color-only status indicators
4. Replace emoji with SVGs + `aria-label` or `aria-hidden`

---

### 3. Architecture & Code Quality 🏗️

| Rating | Finding |
|--------|---------|
| 🔴 | **`repositories.rs` is 7,213 lines** — god file mixing all domains |
| 🔴 | **`App.tsx` is 3,043 lines** — monolithic root component |
| 🔴 | **SQL injection vectors** via `format!` string interpolation |
| 🔴 | **No error boundaries** anywhere in the codebase |
| 🟡 | Error type erosion: `Result<T, String>` → generic 500s |
| 🟡 | In-process-only alert cache (no distributed cache) |
| 🟡 | Naive CSV `split(',')` parsing |
| 🟡 | Raw column name interpolation in `count_school_field` |
| 🟡 | Migration debt from table rebuilds |
| 🟡 | Stub `loadLectureSessions` TODO |
| 🟡 | Inline dynamic type imports causing drift |
| 🟢 | Thin route layer with good auth middleware |
| 🟢 | Clean Tauri-to-HTTP dispatch table (`api.ts`) |
| 🟢 | Data-preserving migration discipline |
| 🟢 | Explicit ticket status transition validation |

**P0 Actions:**
1. Split `repositories.rs` into domain-specific modules
2. Extract `App.tsx` state into context/store
3. Replace all `format!` SQL with parameterized queries
4. Add Error Boundaries to major view areas

---

### 4. Performance & Scalability ⚡

| Rating | Finding |
|--------|---------|
| 🔴 | **N+1 queries** in hot paths (400 queries for faculty utilization) |
| 🔴 | **Correlated subqueries** in Control Tower (4 per school row) |
| 🔴 | **Unbounded result sets** — `refresh_escalations` loads ALL tickets |
| 🔴 | **495KB monolithic JS bundle** — no code splitting |
| 🔴 | **SQLite single-writer ceiling** — 10-connection pool with concurrent writes |
| 🔴 | **Unbounded in-memory caches** — `ALERT_CACHE` HashMap grows forever |
| 🟡 | Query plan cache poisoning from `IN` clause string interpolation |
| 🟡 | 60s alert polling without backoff or tab visibility check |
| 🟡 | ~15 API calls fired simultaneously on mount |
| 🟡 | No frontend request deduplication |
| 🟡 | Synchronous DB in async handlers (no `spawn_blocking`) |
| 🟢 | Good baseline indexes on tickets, students, audit_log |
| 🟢 | 30-second alert TTL cache |
| 🟢 | `LIMIT 1000` on most list endpoints |
| 🟢 | WAL mode enabled |

**P0 Actions:**
1. Add composite indexes: `lecture_sessions(session_date, status)`, `timetable_weekly_slots(school_id, week_start_date, day_of_week, period, faculty_user_id)`
2. Rewrite `refresh_escalations` to single `UPDATE ... WHERE`
3. Cap `ALERT_CACHE` with LRU (50 entries)
4. Implement React.lazy + Suspense for dashboard panels

---

### 5. Database & Data Integrity 🗄️

| Rating | Finding |
|--------|---------|
| 🔴 | **Aggregation Cartesian product** in `faculty_stability` (1,380× inflation) |
| 🔴 | **Attendance aggregation inflation** — counts multiplied by periods |
| 🔴 | **SQL injection** in `week_diff` via string interpolation |
| 🟡 | Migration execution order non-sequential (49,50,51 before 44,45,46) |
| 🟡 | `refresh_escalations` row-by-row updates without transaction |
| 🟡 | `tickets.school_id` lacks foreign key constraint |
| 🟡 | `lecture_sessions` nullable `timetable_slot_id` weakens integrity |
| 🟡 | Denormalized data in `tickets` and `timetable_slots` |
| 🟡 | `control_tower` O(n²) attendance subquery |
| 🟡 | `check_core_subject_gaps` expensive CROSS JOIN (39K rows min) |
| 🟢 | Data-preserving migrations (no DROP COLUMN) |
| 🟢 | Idempotency via `run_migration` + `migration_applied` |
| 🟢 | CHECK constraints on recent tables |
| 🟢 | Soft deletes with partial index |
| 🟢 | WAL + foreign_keys ON |
| 🟢 | Audit trail (`audit_log`, `ticket_history`) |

**P0 Actions:**
1. Rewrite `faculty_stability` to avoid multi-join Cartesian product
2. Fix `attendance_summary` grouping to use session-level granularity
3. Parameterize ALL `analytics.rs` queries

---

### 6. Mobile & Frontend Engineering ⚛️

| Rating | Finding |
|--------|---------|
| 🔴 | **`App.tsx` is 3,109-line god component** — ~60 useState, ~40 loaders, 130-line ternary render chain |
| 🔴 | **No error boundaries** — any exception white-screens the app |
| 🔴 | **No API retry / cancellation** — silent `catch { /* ignore */ }` in 20+ locations |
| 🔴 | **6,491-line monolithic CSS file** — desktop-first, no scoping, collision risk |
| 🔴 | **`visibleTickets` computed on every render** — no useMemo, causes jank |
| 🟡 | Extreme prop drilling — Sidebar receives ~45 callback props |
| 🟡 | Type-safety erosion — `Record<string, unknown>` inputs, unsafe casts |
| 🟡 | No URL state / routing — `AdminView` string union, no React Router |
| 🟡 | Missing AbortController — stale responses overwrite fresh state |
| 🟡 | Hardcoded DOM queries — `document.querySelector('input[aria-label="Search tickets"]')` |
| 🟢 | Strong mobile UX — safe-area insets, 44px touch targets, iOS zoom prevention |
| 🟢 | Comprehensive types.ts (1,157 lines) — discriminated unions, advanced TS |
| 🟢 | `useModalFocus` hook — focus trapping where used |
| 🟢 | PWA foundations — viewport meta, manifest link, theme-color |
| 🟢 | Offline cache hook — localStorage-based with sync detection |

**P0 Actions:**
1. Add Error Boundaries around major views
2. Memoize `visibleTickets` with `useMemo`
3. Replace all silent catches with error feedback
4. Add API retry with exponential backoff

---

### 7. Backend & API Design 🔌

| Rating | Finding |
|--------|---------|
| 🔴 | **SQL injection in `automation.rs`** — `hours` interpolated into SQL |
| 🔴 | **`update_subject` ignores Path parameter** — uses body `id`, auth bypass risk |
| 🔴 | **Missing auth on `POST /notifications`** — any user can spam any other user |
| 🔴 | **`Paginated<T>` defined but completely unused** — all lists return raw `Vec<T>` |
| 🔴 | **No API versioning** — all routes under `/api/`, no `/v1/` |
| 🔴 | **Zero API documentation** — no OpenAPI, no rustdoc |
| 🟡 | Action-oriented URLs mixed with resource-oriented (`/drop`, `/restore`, `/clone`) |
| 🟡 | Inconsistent sub-resource nesting (comments under tickets + top-level) |
| 🟡 | Missing single-resource GET endpoints (`GET /tickets/:id`) |
| 🟡 | No structured validation — relies solely on serde deserialization |
| 🟡 | Repository `String` errors always become 500s |
| 🟡 | No panic-catch middleware |
| 🟡 | No request logging/trace layer |
| 🟡 | No global rate limiting |
| 🟢 | Auth middleware is clean and centralized |
| 🟢 | Error response format is uniform `{ "error": "..." }` |
| 🟢 | Frontend contract explicit via `api.ts` dispatch table |
| 🟢 | School scoping consistently applied |

**P0 Actions:**
1. Fix SQL injection in `automation.rs`
2. Fix `update_subject` to use `Path(id)`
3. Add role check to `POST /notifications`

---

### 8. Product & Feature Strategy 🎯

| Dimension | Rating | Key Reason |
|-----------|--------|------------|
| User Workflow Completeness | 🟡 Missing | Faculty complete; admin roles lack action closure |
| Feature Depth | 🟡 Missing | Backend depth strong; frontend often stubs |
| User Role Appropriateness | 🟡 Missing | Mobile views exist but desktop defaults misaligned |
| Integration Points | 🔴 Critical Gap | Policy/escalation/alert integrations broken or unbuilt |
| Data-Driven Decision Making | 🟡 Missing | Actionable scorecards exist, no persistence/export/drill-down |
| Competitive Positioning | 🟡 Missing | Strong mobile + auto-suggest; weak real-time + predictive |
| Roadmap Readiness | 🟢 Complete | Clear dependency chain; most P0s are wiring, not invention |

**Critical Finding — "Configuration Graveyard":**
- `PolicyConfigPanel` lets users set `max_periods_per_faculty` and `attendance_marking_deadline`, but these values are **not read by any other module**
- `EscalationRulesPanel` stores JSON conditions, yet `refresh_escalations` still uses hardcoded legacy policy
- Weekly digest content generates but has no SMTP sender
- Historical snapshot tables required for trend charts were never created
- `alert_states` table exists but `AlertInboxPanel` cannot persist dismissals

**Bottom line:** SAATHI is a feature-rich platform with a **solid foundation and a wiring problem**. The highest ROI work is finishing integration seams, not adding new modules.

---

## Recommendations by Priority

### 🔴 P0 — Fix Before Production (Safety, Correctness, Security)

| # | Issue | Owner | Effort | Files |
|---|-------|-------|--------|-------|
| P0.1 | Parameterize all SQL queries (eliminate `format!` interpolation) | Backend | 2 days | `analytics.rs`, `alerts.rs`, `repositories.rs`, `automation.rs` |
| P0.2 | Fix SQL injection in `automation.rs::bulk_alert_action` | Backend | 0.5 days | `automation.rs` |
| P0.3 | Fix `update_subject` to use `Path(id)` | Backend | 0.5 days | `faculty.rs` |
| P0.4 | Add role check to `POST /notifications` | Backend | 0.5 days | `notifications.rs` |
| P0.5 | Fix `faculty_stability` Cartesian product | Backend | 1 day | `analytics.rs` |
| P0.6 | Fix `attendance_summary` aggregation inflation | Backend | 1 day | `repositories.rs` |
| P0.7 | Implement JWT token expiry + refresh | Backend | 2 days | `auth.rs`, `api.ts` |
| P0.8 | Add authorization checks to unguarded Phase 6 routes | Backend | 1 day | `routes/*.rs` |
| P0.9 | Add Error Boundaries to major views | Frontend | 1 day | `App.tsx` |
| P0.10 | Memoize `visibleTickets` with `useMemo` | Frontend | 0.5 days | `App.tsx` |
| P0.11 | Stop silencing errors — replace `catch { /* ignore */ }` | Frontend | 1 day | Multiple files |
| P0.12 | Add `role="dialog"` and focus trap to all modals | Frontend | 1 day | Modal components |
| P0.13 | Enlarge touch targets to 44px | Frontend | 0.5 days | `styles.css` |
| P0.14 | Add non-color cues to status indicators | Frontend | 0.5 days | Dashboard components |

### 🟡 P1 — Fix Before Scale (Performance, Maintainability)

| # | Issue | Owner | Effort | Files |
|---|-------|-------|--------|-------|
| P1.1 | Add composite indexes for hot query paths | Backend | 0.5 days | `db.rs` |
| P1.2 | Rewrite `refresh_escalations` to batch UPDATE | Backend | 1 day | `repositories.rs` |
| P1.3 | Cap `ALERT_CACHE` with LRU | Backend | 0.5 days | `alerts.rs` |
| P1.4 | Implement React.lazy + Suspense | Frontend | 2 days | `App.tsx` |
| P1.5 | Extract App.tsx state to context/store | Frontend | 3 days | `App.tsx` |
| P1.6 | Add request deduplication to `api.ts` | Frontend | 0.5 days | `api.ts` |
| P1.7 | Re-order migrations sequentially | Backend | 0.5 days | `db.rs` |
| P1.8 | Add transaction wrapper to `refresh_escalations` | Backend | 0.5 days | `repositories.rs` |
| P1.9 | Add `FOREIGN KEY` to `tickets.school_id` | Backend | 1 day | `db.rs` |
| P1.10 | Role-based default views | Frontend | 1 day | `App.tsx` |
| P1.11 | Wire `policies.rs` into `alerts.rs` | Backend | 1 day | `alerts.rs` |
| P1.12 | Wire `escalation.rs` into ticket refresh | Backend | 1 day | `tickets.rs` |
| P1.13 | Introduce API versioning (`/api/v1/`) | Backend | 1 day | `mod.rs`, `api.ts` |
| P1.14 | Add structured input validation (`validator` crate) | Backend | 2 days | `models.rs`, `routes/*.rs` |

### 🟢 P2 — Polish & Strategic

| # | Issue | Owner | Effort | Files |
|---|-------|-------|--------|-------|
| P2.1 | Skeleton screens for slow panels | Frontend | 2 days | Dashboard components |
| P2.2 | Add search to MobileMoreMenu | Frontend | 1 day | `MobileMoreMenu.tsx` |
| P2.3 | Unify primary action color | Frontend | 0.5 days | `styles.css` |
| P2.4 | Distinguish empty states from errors | Frontend | 0.5 days | `FacultyApp.tsx` |
| P2.5 | Add `session_type` discriminator to `lecture_sessions` | Backend | 1 day | `db.rs`, `models.rs` |
| P2.6 | API response caching (5-min TTL) for analytics | Backend | 2 days | `analytics.rs` |
| P2.7 | Pagination for tickets, students, audit logs | Backend | 2 days | `routes/*.rs` |
| P2.8 | Wrap DB-heavy handlers in `spawn_blocking` | Backend | 1 day | `routes/*.rs` |
| P2.9 | Historical snapshot tables + cron | Backend | 3 days | `db.rs`, new module |
| P2.10 | CSV export for all dashboards | Frontend | 1 day | Dashboard components |
| P2.11 | Add `react-router-dom` for URL routing | Frontend | 2 days | `App.tsx` |
| P2.12 | Add `tower_http::TraceLayer` + `CatchPanicLayer` | Backend | 0.5 days | `main.rs` |
| P2.13 | Add global rate-limiting middleware | Backend | 1 day | `main.rs` |
| P2.14 | Generate OpenAPI spec | Backend | 2 days | New module |

---

## Overall Committee Verdict

**🟡 CONDITIONAL DEPLOY**

SAATHI is a feature-rich, well-intentioned platform with strong foundations (Rust/Axum, React, data-preserving migrations, bcrypt auth, WAL mode). The faculty mobile experience is genuinely competitive. The backend analytics engine is production-grade.

However, **critical correctness issues** in analytics queries, **security gaps** in Phase 6 routes, and **accessibility failures** must be addressed before production deployment. The app is **not ready for 50+ schools** without the P1 performance fixes.

The single biggest strategic risk is the **"integration seam" problem** — substantial backend code exists but is not wired to drive behavior. Policies, escalation rules, and alert engines are "configuration graveyards" (UI without effect).

**Estimated time to production-ready:** 2–3 weeks of focused engineering on P0 + P1 items.

**Highest ROI work:** Finish wiring existing code rather than adding new modules. Move from "dashboard museum" to "operations command center."

---

*Report compiled from 8 independent expert reviews.*
*Files reviewed: 24 backend modules, 65+ frontend components, 55 DB migrations, 89 API endpoints.*
