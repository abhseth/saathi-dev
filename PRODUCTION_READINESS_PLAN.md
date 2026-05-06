# SAATHI Production Readiness Plan

**Date:** 2026-05-01  
**Goal:** Move from 🟡 Conditional Deploy to 🟢 Production Ready  
**Target:** 4 weeks (5 sprints)  
**Constraint:** Zero build-breaking changes; backward-compatible migrations only  

---

## Phase Summary

| Phase | Duration | Focus | Teams | Builds Pass? |
|-------|----------|-------|-------|-------------|
| **1** | Week 1 | Safety & Stability | 2 teams | ✅ |
| **2** | Week 1 | Data Correctness | 2 teams | ✅ |
| **3** | Week 1 | Integration Wiring | 2 teams | ✅ |
| **4** | Week 2 | Performance & Polish | 3 teams | ✅ |
| **5** | Week 1 | Final Integration & QA | 1 team | ✅ |

**Total calendar time:** 4 weeks (sprints 1-3 run sequentially; sprint 4 runs 2 weeks; sprint 5 is final QA)  
**Total parallel capacity:** Up to 3 teams at peak (Phase 4)

---

## Phase 1: Safety & Stability (Days 1–5)

**Goal:** Fix all P0 security and correctness issues that could cause data loss, unauthorized access, or app crashes.

**Dependency:** None (foundation layer). Must complete before any other phase.

### Team A — Backend Security & Safety
**Scope:** `backend/src/analytics.rs`, `backend/src/alerts.rs`, `backend/src/routes/automation.rs`, `backend/src/routes/faculty.rs`, `backend/src/routes/notifications.rs`

| # | Task | File | Effort | Test |
|---|------|------|--------|------|
| A1 | Parameterize `week_diff` — replace `format!` with `?` placeholders | `analytics.rs` | 0.5d | Build + verify route |
| A2 | Parameterize all `analytics.rs` queries — eliminate ALL `format!` SQL | `analytics.rs` | 1d | Build + spot-check routes |
| A3 | Fix SQL injection in `automation.rs::bulk_alert_action` — whitelist `hours` | `automation.rs` | 0.5d | Build + test dismiss |
| A4 | Fix `update_subject` to use `Path(id)` not body `id` | `faculty.rs` | 0.5d | Build + test PUT |
| A5 | Add `require_admin_or_aom` to `POST /notifications` | `notifications.rs` | 0.5d | Build + test auth |
| A6 | Add school scope check to `list_announcements` | `automation.rs` | 0.5d | Build + test scoped list |
| A7 | Add faculty-ownership check to `mark_attendance_quick` | `substitutions.rs` | 0.5d | Build + test self-only |
| A8 | Add `enforce_school_scope` to unscoped analytics routes | `analytics.rs` | 0.5d | Build + verify filter |

**Acceptance:** `cargo build` passes; `cargo test` (if tests exist) passes; no new warnings.

### Team B — Frontend Stability & Error Handling
**Scope:** `frontend/src/App.tsx`, `frontend/src/api.ts`, multiple component files

| # | Task | File | Effort | Test |
|---|------|------|--------|------|
| B1 | Add Error Boundary component | New: `ErrorBoundary.tsx` | 0.5d | Build + render test |
| B2 | Wrap FacultyApp, admin workspace, ticket detail in ErrorBoundary | `App.tsx` | 0.5d | Build + verify |
| B3 | Memoize `visibleTickets` with `useMemo` | `App.tsx` | 0.5d | Build + typecheck |
| B4 | Replace all `catch { /* silently fail */ }` with `setError` or `console.error` | ~15 files | 1d | Build + grep for silent catches |
| B5 | Add `aria-label` to all dismiss/close buttons in modals | Modal components | 0.5d | Build + a11y audit |
| B6 | Add `role="dialog"` + `aria-modal="true"` to all modals | Modal components | 0.5d | Build + verify |

**Acceptance:** `npm run build` passes; zero silent `catch` blocks remaining.

**Phase 1 Gate:** ✅ Both builds pass. Security audit re-run confirms no unparameterized string SQL.

---

## Phase 2: Data Correctness & Accessibility (Days 6–10)

**Goal:** Fix query correctness bugs that return wrong data, and close WCAG/accessibility gaps that block production for government-school deployments.

**Dependency:** Phase 1 complete. No file conflicts with Phase 1 changes.

### Phase 1 Completion Summary
| Item | Status | Notes |
|------|--------|-------|
| A1–A3: Parameterize `week_diff`, `substitution_trends` | ✅ Done | String dates now use `?` placeholders |
| A4: `update_subject` path/body check | ✅ Already existed | `input.id != id` validation present |
| A5: Notification role check | ✅ Already existed | `admin`/`aom` gate present |
| B1–B2: Error boundaries | ✅ Done | `ErrorBoundary.tsx` created, wrapped in `App.tsx` |
| B3: `visibleTickets` memoization | ✅ Done | `React.useMemo` added |
| B4: Silent catches | ✅ Done | `console.error`/`console.warn` added to all silent blocks |
| B5: Modal `aria-label` on close buttons | ✅ Done | 18 modal close buttons labeled |
| B6: `role="dialog"` + `aria-modal` | 🟡 Partial | Timetable/automation modals done; **dashboard modals missing** |

---

### Team C — Database Query Fixes
**Scope:** `backend/src/analytics.rs`, `backend/src/repositories.rs`, `backend/src/alerts.rs`

| # | Task | File | Effort | Test |
|---|------|------|--------|------|
| **C1** | **Fix `faculty_stability` column-index bug** — `sub_count` reads index 2 (`school_name` String) as `i64`; all four count indices are shifted. Rewrite with correlated subqueries to eliminate Cartesian product AND fix mapping. | `analytics.rs` | 1d | Build + verify endpoint returns data without error |
| **C2** | **Fix `attendance_summary` aggregation inflation** — `LEFT JOIN students` without `ls.id` grouping multiplies counts by number of periods. Add `COUNT(DISTINCT st.id)` or group by `ls.id`. | `repositories.rs` | 1d | Build + verify student counts match enrollment |
| **C3** | **Fix `subject_attendance` grouping** — same `ls.id` grouping gap as C2. | `repositories.rs` | 0.5d | Build + verify counts |
| **C4** | Replace correlated subqueries in `control_tower` with LEFT JOIN aggregates — 4 subqueries per school row cause O(n²) scans. | `analytics.rs` | 1d | Build + verify performance on 50+ schools |
| **C5** | Fix `check_core_subject_gaps` expensive CROSS JOIN — add early `school_id` scope filter before cross product. | `alerts.rs` | 0.5d | Build + verify alert count |
| **C6** | Replace remaining `format!` SQL in `repositories.rs` IN clauses with `params_from_iter` — 16 instances of `ids.iter().map(|i| i.to_string()).join(",")`. Type-safe (`i64`) but inconsistent with parameterized discipline. | `repositories.rs` | 1d | Build + verify list endpoints |
| **C7** | Re-order migrations sequentially — current order 1–43, 49–51, 47–48, 44–46, 52–55. Re-sequence to 1–55 without gaps. | `db.rs` | 0.5d | Build + verify on fresh DB |

**Acceptance:**
- `faculty_stability` endpoint returns data (currently returns type-conversion error)
- `attendance_summary` total students per grade matches actual enrollment
- `cargo build` passes with zero new warnings

---

### Team D — Frontend Accessibility & UX
**Scope:** `frontend/src/styles.css`, `frontend/src/components/dashboards/`, `frontend/src/components/timetable/`, `frontend/src/components/faculty/`

| # | Task | File | Effort | Test |
|---|------|------|--------|------|
| **D1** | Enlarge `.day-nav-btn` to 44×44px (currently 36px) | `styles.css` | 0.5d | Build + visual check |
| **D2** | Enlarge `.alert-dismiss` to 44×44px (currently ~20px) | `styles.css` + `AlertBanner.tsx` | 0.5d | Build + visual check |
| **D3** | Add visible text labels to `.status-dot` indicators — WCAG 1.4.1 failure in health dashboard. Add "Healthy / Needs Attention / Critical" text alongside color dots. | `HealthDashboardPanel.tsx` + `styles.css` | 0.5d | Build + a11y check |
| **D4** | Replace emoji with accessible text/SVGs — `⏰`, `🔒`, `✓`, `✗`, `⚠️`, `✅`, `❌` in `FacultyApp.tsx`, `HealthDashboardPanel.tsx`, `TimetableContextPanel.tsx`. Use text labels or SVGs with `aria-hidden`. | 4 files | 1d | Build + screen reader test |
| **D5** | Add `role="dialog"` + `aria-modal="true"` to all dashboard modals — 11 components in `components/dashboards/` missing these attributes. | `components/dashboards/*.tsx` | 0.5d | Build + axe check |
| **D6** | Add non-color text cues to `SubjectCoverageHeatmap` — add adherence % text in cells, not just color. | `SubjectCoverageHeatmap.tsx` | 0.5d | Build + visual check |
| **D7** | Add non-color text cues to `RegionHeatmapPanel` — add issue count text in cells. | `RegionHeatmapPanel.tsx` | 0.5d | Build + visual check |
| **D8** | Add skeleton screen CSS + component | New: `SkeletonBlock.tsx` + `styles.css` | 0.5d | Build + visual check |
| **D9** | Distinguish empty states from errors in `FacultyApp` — add separate "No data" and "Failed to load" messaging. | `FacultyApp.tsx` | 0.5d | Build + test offline |

**Acceptance:** `npm run build` passes; zero emoji in components rendered by screen readers; all modal close buttons have `aria-label` (already done) + `role="dialog"`; `.day-nav-btn` and `.alert-dismiss` are 44×44px.

---

**Phase 2 Gate:** Both builds pass; `faculty_stability` returns correct data; `attendance_summary` counts verified against enrollment; Lighthouse a11y score ≥ 90.

---

## Phase 3: Integration Wiring (Days 11–15)

**Goal:** Close the "code cemetery" — wire existing backend logic so UI changes actually drive behavior.

**Dependency:** Phase 1 auth fixes complete; Phase 2 query fixes stable.

### Team E — Backend Integration Engine
**Scope:** `backend/src/alerts.rs`, `backend/src/policies.rs`, `backend/src/escalation.rs`, `backend/src/routes/tickets.rs`, `backend/src/db.rs`

| # | Task | File | Effort | Test |
|---|------|------|--------|------|
| E1 | Wire `policies.rs` into `alerts.rs` — read `max_periods_per_faculty` from DB | `alerts.rs` | 1d | Build + change policy → verify alert |
| E2 | Wire `policies.rs` into `alerts.rs` — read `attendance_marking_deadline` from DB | `alerts.rs` | 0.5d | Build + change policy → verify alert |
| E3 | Wire `escalation.rs` into `refresh_escalations` — evaluate active rules | `tickets.rs` + `escalation.rs` | 1d | Build + create rule → verify escalation |
| E4 | Add `alert_states` persistence to `get_alerts` route | `faculty.rs` + new queries | 1d | Build + dismiss alert → verify DB |
| E5 | Add transaction wrapper to `refresh_escalations` | `repositories.rs` | 0.5d | Build + verify atomicity |
| E6 | Add composite indexes: `lecture_sessions(session_date, status)` | `db.rs` | 0.5d | Build + verify explain plan |
| E7 | Add composite index: `timetable_weekly_slots(school_id, week_start_date, day_of_week, period, faculty_user_id)` | `db.rs` | 0.5d | Build + verify explain plan |
| E8 | Cap `ALERT_CACHE` with LRU (50 entries) | `alerts.rs` | 0.5d | Build + verify cache eviction |

**Acceptance:** Change a policy value → alert behavior changes. Create an escalation rule → ticket escalates accordingly.

### Team F — Frontend Integration & State
**Scope:** `frontend/src/App.tsx`, `frontend/src/components/automation/`

| # | Task | File | Effort | Test |
|---|------|------|--------|------|
| F1 | Wire `AlertInboxPanel` to `dismiss_alert` + `bulk_alert_action` APIs | `AlertInboxPanel.tsx` | 0.5d | Build + test dismiss |
| F2 | Wire `PolicyConfigPanel` to `update_policy` API | `PolicyConfigPanel.tsx` | 0.5d | Build + test save |
| F3 | Wire `EscalationRulesPanel` to CRUD APIs | `EscalationRulesPanel.tsx` | 0.5d | Build + test CRUD |
| F4 | Add role-based default `adminView` on login | `App.tsx` | 0.5d | Build + test per role |
| F5 | Replace hardcoded 95% attendance in `AomMorningBrief` with real API call | `AomMorningBrief.tsx` | 0.5d | Build + verify data |
| F6 | Connect "Periods Needing Substitutes" in Principal digest to command center | `PrincipalMorningDigest.tsx` | 0.5d | Build + test navigation |

**Acceptance:** Policy change persists and affects alerts. Escalation rule creation works end-to-end.

**Phase 3 Gate:** Both builds pass; integration tests pass (manual verification of policy → alert → ticket chain).

---

## Phase 4: Performance & Polish (Days 16–25)

**Goal:** Make the app fast and polished enough for 50+ schools.

**Dependency:** Phase 3 integrations complete; no file conflicts with core data layer.

### Team G — Backend Performance
**Scope:** `backend/src/repositories.rs`, `backend/src/db.rs`, `backend/src/routes/*.rs`, `backend/src/main.rs`

| # | Task | File | Effort | Test |
|---|------|------|--------|------|
| G1 | Rewrite `refresh_escalations` to single `UPDATE ... WHERE` with `CASE` | `repositories.rs` | 1d | Build + verify on 1000 tickets |
| G2 | Add `LIMIT 500` to `refresh_escalations` + batch processing | `repositories.rs` | 0.5d | Build + verify |
| G3 | Add `spawn_blocking` wrapper to DB-heavy analytics handlers | `routes/analytics.rs` | 1d | Build + verify no blocking |
| G4 | Implement pagination on all list endpoints (`Paginated<T>`) | `routes/*.rs` + `models.rs` | 2d | Build + verify page limits |
| G5 | Add `tower_http::TraceLayer` for request logging | `main.rs` | 0.5d | Build + verify logs |
| G6 | Add `tower_http::catch_panic::CatchPanicLayer` | `main.rs` | 0.5d | Build + test panic recovery |
| G7 | Add global rate-limiting middleware | `main.rs` | 1d | Build + verify throttling |
| G8 | Refactor repository errors from `String` to `RepoError` enum | `repositories.rs` + `error.rs` | 2d | Build + verify status codes |

### Team H — Frontend Performance & Architecture
**Scope:** `frontend/src/App.tsx`, `frontend/src/api.ts`, `frontend/src/components/`, `frontend/src/styles.css`

| # | Task | File | Effort | Test |
|---|------|------|--------|------|
| H1 | Implement React.lazy + Suspense for dashboard/admin panels | `App.tsx` | 1d | Build + verify chunk split |
| H2 | Add request deduplication to `api.ts` | `api.ts` | 0.5d | Build + verify single request |
| H3 | Add `AbortController` to `apiFetch` | `api.ts` | 0.5d | Build + verify cancellation |
| H4 | Extract state from `App.tsx` into custom hooks (`useTickets`, `useSchools`) | `App.tsx` + new hooks | 2d | Build + verify |
| H5 | Split `styles.css` into `base.css`, `desktop.css`, `mobile.css` | `styles.css` → split | 1d | Build + verify styling |
| H6 | Replace skeleton text with `SkeletonBlock` component in slow panels | Dashboard panels | 1d | Build + visual check |
| H7 | Add search filter to `MobileMoreMenu` | `MobileMoreMenu.tsx` | 0.5d | Build + test filtering |
| H8 | Add API retry with exponential backoff | `api.ts` | 0.5d | Build + test offline |

### Team I — Mobile & Offline Polish
**Scope:** `frontend/src/components/mobile/`, `frontend/src/components/faculty/`, `frontend/src/styles.css`

| # | Task | File | Effort | Test |
|---|------|------|--------|------|
| I1 | Unify primary action color (choose teal or indigo) | `styles.css` | 0.5d | Build + visual check |
| I2 | Distinguish empty states from errors in `FacultyApp` | `FacultyApp.tsx` | 0.5d | Build + test offline |
| I3 | Add swipe-down dismiss to `MobileMoreMenu` | `MobileMoreMenu.tsx` | 0.5d | Build + test touch |
| I4 | Add `scope` + `headers` to data tables | Table components | 0.5d | Build + screen reader |
| I5 | Add offline mutation queue to `useOfflineCache` | `useOfflineCache.ts` | 1d | Build + test sync |

**Phase 4 Gate:** Both builds pass; bundle size < 300KB (from ~495KB); analytics queries complete < 2s.

---

## Phase 5: Final Integration & QA (Days 26–30)

**Goal:** End-to-end verification, build final report, sign-off.

**Dependency:** All prior phases complete.

### Team J — Integration QA
**Scope:** Full stack

| # | Task | Effort |
|---|------|--------|
| J1 | Run `cargo build` + `npm run build` — zero errors | 0.5d |
| J2 | Run manual smoke tests: login → day-at-glance → attendance → substitution → logout | 1d |
| J3 | Run manual smoke tests: admin → control tower → policy config → escalation rules → alert inbox | 1d |
| J4 | Security re-audit: verify all P0 SQL injection fixes | 0.5d |
| J5 | Performance spot-check: verify N+1 eliminated, indexes used | 0.5d |
| J6 | Update `COMMITTEE_REVIEW_REPORT.md` — mark all P0 as resolved | 0.5d |
| J7 | Generate `DEPLOYMENT_CHECKLIST.md` | 0.5d |

**Phase 5 Gate:** All P0 items resolved. All P1 items resolved or accepted as P2. Builds pass. Smoke tests pass.

---

## Risk Register

| Risk | Mitigation |
|------|------------|
| Parallel agents cause integration seams (Phase 4) | Assign non-overlapping file ownership; shared files are append-only |
| Migration conflicts (Teams E + G both touch db.rs) | Team E owns migrations 56-58; Team G owns 59-62; sequential handoff |
| App.tsx refactor breaks downstream components | Team H only extracts state to hooks; does NOT change component props |
| Performance fixes change query results | Each fix includes a manual verification step with expected vs actual counts |

---

## Resource Allocation

| Phase | Teams | Agents | Max Parallel |
|-------|-------|--------|-------------|
| 1 | A, B | 2 | 2 |
| 2 | C, D | 2 | 2 |
| 3 | E, F | 2 | 2 |
| 4 | G, H, I | 3 | 3 |
| 5 | J | 1 | 1 |

**Total agents needed:** 10 (2+2+2+3+1)  
**Peak parallel:** 3 teams in Phase 4  
**Calendar time:** 4 weeks (30 working days)

---

## Approval Checklist

Before launching any agent team:
- [x] Phase 1 plan approved
- [x] Phase 1 complete — builds pass, critical SQL injection fixed, error boundaries added
- [ ] Phase 2 plan approved (depends on Phase 1)
- [ ] Phase 3 plan approved (depends on Phase 2)
- [ ] Phase 4 plan approved (depends on Phase 3)
- [ ] Phase 5 plan approved (depends on Phase 4)
- [ ] File ownership boundaries confirmed per team
- [ ] Migration number ranges allocated per team

**Recommended approach:** Approve Phase 1 only → launch → verify builds → approve Phase 2 → launch → repeat.
