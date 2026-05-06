# Critical Review Council Report — Timetable Implementation

**Date:** 2026-05-01  
**Council:** 5 specialized review agents  
**Scope:** Phase 5 Timetable Views (Backend + Frontend + Integration)  
**Overall Verdict:** 🔴 **Do Not Deploy — Critical Issues Found**

---

## Executive Summary

The timetable implementation delivers **93% of stakeholder requirements** but the **integration layer is broken**. Four critical issues would cause immediate production failures:

1. **Migration 42 is duplicated** — ticket linking columns never created on fresh databases
2. **Every new API has URL mismatches** — frontend calls `/timetable/health`, backend serves `/timetable-health` → 404 for all Phase 5 features
3. **Response type shapes are incompatible** — frontend expects nested aggregates, backend returns flat rows
4. **Alert engine is unscoped and uncached** — every user poll triggers 6 full-table scans

5. **`day_of_week` convention mismatch** — `DayAtAGlancePanel` uses Monday=1..Sunday=7 (`getDay()`), DB stores Monday=0. Today's slots never match.
6. **Infinite re-render loops in 4 panels** — `loadXxx` callbacks in `App.tsx` not wrapped in `useCallback`, causing continuous backend hammering.

**Bottom line:** The code compiles and the UI renders, but **no Phase 5 feature actually works end-to-end**. The gaps are all at the integration seams between agents.

---

## 1. Critical Blockers (Ship-Stoppers)

### 🔴 B1: Duplicate Migration 42 — Database Schema Broken

**Found by:** Security Reviewer, Database Reviewer, Architecture Reviewer  
**File:** `backend/src/db.rs:979–1018`

Two migration blocks claim version `42`:
- Block 1 (lines 979–1001): Adds `room`/`session_type` to timetable slots, inserts `42` into `schema_migrations`
- Block 2 (lines 1003–1018): Adds `linked_grade_level`/`linked_subject` to tickets

**Impact:** Block 2 is **dead code**. On fresh installs, `tickets.linked_grade_level` and `tickets.linked_subject` **do not exist**. All `SELECT` queries referencing these columns will fail with `"no such column"`.

**Fix:** Move ticket linking columns to **Migration 43**. On existing databases where 42 already ran, add a forward-migration for 43.

---

### 🔴 B2: Frontend ↔ Backend API Path Mismatch — All New Features 404

**Found by:** Security Reviewer, Architecture Reviewer  
**Files:** `frontend/src/api.ts`, `backend/src/routes/mod.rs`

| Feature | Frontend Calls | Backend Serves | Result |
|---------|---------------|----------------|--------|
| Health Dashboard | `GET /timetable/health` | `GET /timetable-health` | 404 |
| Compliance Analytics | `GET /timetable/compliance` | `GET /compliance-metrics` | 404 |
| Deviation Score | `GET /timetable/deviation-score?school_id=` | `GET /deviation-score/:school_id` | 404 |
| Cross-School Faculty | `GET /faculty/schedule?faculty_user_id=` | `GET /faculty-schedule/:faculty_user_id` | 404 |
| Substitutions | `GET /faculty/substitutions` | `GET /substitutions` | 404 |
| Room Conflicts | `GET /timetable/room-conflicts` | `GET /room-conflicts` | 404 |
| Accept Substitution | `POST /faculty/substitutions/:id/accept` | **Not implemented** | 404 |
| Decline Substitution | `POST /faculty/substitutions/:id/decline` | **Not implemented** | 404 |

**Impact:** Every Phase 5 panel shows "No data" or empty states. The substitution inbox accept/decline buttons are non-functional.

**Fix:** Pick one URL convention and apply uniformly. Implement `accept_substitution` and `decline_substitution` backend routes.

---

### 🔴 B3: Response Type Shape Mismatches — Data Cannot Render

**Found by:** Architecture Reviewer  
**Files:** `backend/src/models.rs`, `frontend/src/types.ts`

| Type | Backend Returns | Frontend Expects | Impact |
|------|-----------------|------------------|--------|
| `TimetableHealthStatus` | `gaps_count: i64`, no `gap_details` | `gaps_found: boolean`, `gap_details: string[]` | Expandable gap rows always show "No gaps" |
| `ComplianceMetrics` | Flat per-subject row | Nested aggregate with `subject_coverage_gaps[]`, `faculty_overloads[]` | Compliance panel cannot render; structure is completely wrong |
| `SubstitutionRecord` | `session_id`, no `id` or `period` | `id`, `period`, `room`, `created_at` | Frontend references non-existent fields |
| `WeeklyTimetableSlot` | Has `room`, `session_type` | Missing both in `types.ts` | Ugly `(s as unknown as Record<string, string>).room` casts |

**Fix:** Align `frontend/src/types.ts` with `backend/src/models.rs`. Add backend DTOs if frontend needs a different shape.

---

### 🔴 B4: Alert Engine Unscoped + Uncached — Scalability Time Bomb

**Found by:** Security Reviewer, Database Reviewer  
**File:** `backend/src/alerts.rs`

All 6 alert checks run **globally** (full table scans), then filter in Rust memory:
```rust
alerts.append(&mut check_unfilled_periods(conn)?);    // scans ALL schools
alerts.append(&mut check_double_bookings(conn)?);     // groups ALL schools
// ...
if let Some(ids) = scope_school_ids {
    alerts.retain(|a| ...);  // filtered AFTER query
}
```

**Load model:**
| Active Users | Polls/Min | Heavy Queries/Sec | Impact |
|--------------|-----------|-------------------|--------|
| 20 | 20 | 2 | ✅ Fine |
| 100 | 100 | 10 | ⚠️ CPU spikes |
| 500 | 500 | 50 | 🔴 Saturates SQLite |

**Also:** `check_double_bookings` queries `timetable_slots` (recurring template) instead of `timetable_weekly_slots` (actual week). It misses real-week conflicts.

**Fix:** Pass `scope_school_ids` into each check function. Add in-memory TTL cache (30–60s) or materialize alerts to a snapshot table.

---

### 🔴 B5: `CrossSchoolFacultyPanel` Data Flow Bug — Always Empty

**Found by:** UX Reviewer  
**File:** `frontend/src/components/timetable/CrossSchoolFacultyPanel.tsx:45–48`

After calling `onLoad`, the component explicitly calls `setSlots([])` in **both `.then()` and `.catch()`**:
```tsx
void onLoad(facultyId, weekStart)
  .then(() => setSlots([]))
  .catch(() => setSlots([]));
```

**Impact:** The Center Head always sees "No scheduled slots for this week" regardless of backend data.

**Fix:** Pass loaded slots into `setSlots(data)` or receive slots via props from `App.tsx`.

### 🔴 B6: Infinite Re-Render Loops — 4 Panels Hammer Backend Continuously

**Found by:** Integration & Testing Reviewer  
**File:** `frontend/src/App.tsx` (multiple load functions)

`App.tsx` defines `loadWeeklyTimetable`, `loadTimetableHealth`, `loadComplianceMetrics`, `loadFacultySchedule` as plain functions inside the component. These are new references on every render. The 4 panels call them inside `useEffect` with the callback in the dependency array, causing the effect to fire every render. Since the callbacks set parent state, the parent re-renders, and the cycle repeats.

**Impact:** Every open panel fires API requests in an infinite loop, hammering the backend.

**Fix:** Wrap all `loadXxx` callbacks in `App.tsx` with `React.useCallback`.

---

### 🔴 B7: `day_of_week` Convention Mismatch — Today's Slots Never Match

**Found by:** Integration & Testing Reviewer  
**File:** `frontend/src/components/timetable/DayAtAGlancePanel.tsx:34–39`

The database stores `day_of_week` as Monday=0 (existing app convention). `DayAtAGlancePanel` computes `todayDayOfWeek` as Monday=1..Sunday=7 using `getDay()`-derived logic. **Today's slots will never match the filter.**

**Impact:** Principal's Day-at-a-Glance always shows "No slots match the selected filters" even when slots exist.

**Fix:** Align all new components to Monday=0 convention.

---

## 2. High-Priority Issues (Fix Before First User Test)

### ⚠️ H1: Alert Scope Leak — Global Alerts Visible to All Users

**Found by:** Security Reviewer  
**File:** `backend/src/alerts.rs:314`

```rust
a.school_id.map(|sid| ids.contains(&sid)).unwrap_or(true)
```

Alerts without a `school_id` (e.g., double-bookings) are visible to **every user**, including faculty/viewers with zero schools.

**Fix:** Change `unwrap_or(true)` to `unwrap_or(false)`.

---

### ⚠️ H2: Missing Indexes — Faculty Queries Do Full Table Scans

**Found by:** Database Reviewer  
**File:** `backend/src/db.rs`

| Missing Index | Impact |
|---------------|--------|
| `timetable_weekly_slots(faculty_user_id, week_start_date)` | Full scan on every faculty schedule lookup |
| `timetable_weekly_slots(school_id, week_start_date, day_of_week, room)` | Room conflict self-join is O(N²) |
| `timetable_slots(school_id, grade_level, track, subject_id)` | Compliance CTEs scan unnecessarily |

**Fix:** Add the three composite indexes in Migration 43.

---

### ⚠️ H3: No Rate Limiting on Alert Polling

**Found by:** Security Reviewer  
**File:** `frontend/src/App.tsx:359–373`

Every authenticated user fires `GET /alerts` every 60 seconds. No backend rate limiting. A single malicious user could open 50 tabs and fire 50 req/min.

**Fix:** Add token-bucket rate limiting on `/alerts` and `/faculty/alerts` (e.g., 1 req/min per user).

---

### ⚠️ H4: `update_ticket` Never Refreshes Linked Metadata

**Found by:** Database Reviewer  
**File:** `backend/src/repositories.rs`

If a user edits a ticket title from "Grade 8 Science issue" to "Grade 9 Science issue", `linked_grade_level` remains `"grade 8"` forever.

**Fix:** Re-run `extract_linked_metadata` in `update_ticket`.

---

### ⚠️ H5: `list_room_conflicts` — 36-Column Ordinal SELECT Is Fragile

**Found by:** Architecture Reviewer  
**File:** `backend/src/repositories.rs:6482–6568`

```rust
SELECT ... 36 columns by position ...
```

Any schema addition to `WeeklyTimetableSlot` shifts column indices and corrupts the paired tuple mapping.

**Fix:** Use a CTE returning `id` pairs, then hydrate `WeeklyTimetableSlot` via separate queries.

---

### ⚠️ H6: Notification Spam in FacultyApp

**Found by:** UX Reviewer  
**File:** `frontend/src/components/faculty/FacultyApp.tsx:175–180`

`sendBrowserNotification` fires for **every** pending substitution on **every** render. 5 pending requests = 5 simultaneous notifications, repeated on re-render.

**Fix:** Track "already notified" IDs in `localStorage` or state. Only notify for new requests.

---

### ⚠️ H7: Accessibility — Color-Only Indicators, No Focus Management

**Found by:** UX Reviewer  
**Files:** All new timetable panels

- No focus trap on modal open
- No `Escape` key handler to close modals
- Green/Amber/Red status is color-only (colorblind users cannot distinguish)
- Substitution dot (`●`) has no `aria-label`
- Deviation bars rely solely on color with no text prefix

**Fix:** Add `aria-label` text badges to all color-coded indicators. Implement `useRef` + `useEffect` focus trap.

---

## 3. Medium-Priority Issues (Fix Before Production)

### ⚠️ M1: `check_unfilled_periods` Not Scoped to Today/Current Week

**Found by:** Database Reviewer  
**File:** `backend/src/alerts.rs:27`

Alerts on periods 6 months in the future. Should be scoped to current week.

---

### ⚠️ M2: `check_attendance_not_marked` Missing 11 AM Gate

**Found by:** Database Reviewer  
**File:** `backend/src/alerts.rs:256`

Fires at 9 AM instead of after 11 AM.

---

### ⚠️ M3: `check_double_bookings` Queries Wrong Table

**Found by:** Architecture Reviewer  
**File:** `backend/src/alerts.rs:73`

Queries `timetable_slots` (recurring template) instead of `timetable_weekly_slots` (actual week). Misses real conflicts.

---

### ⚠️ M4: English-Only Keyword Matching

**Found by:** UX Reviewer  
**Files:** `TimetableContextPanel.tsx:6–8`, `repositories.rs:372–400`

Schedule-related tickets in Hindi/Urdu (e.g., *"कक्षा 9 गणित अनुपस्थित"*) will not trigger the context panel or auto-linking.

---

### ⚠️ M5: `session_type` Lacks CHECK Constraint

**Found by:** Database Reviewer  
**File:** `backend/src/db.rs`

No validation. Typos like `'lecture'` or `'Lecure'` will break analytics.

---

### ⚠️ M6: Raise Ticket Button Too Small on Mobile

**Found by:** UX Reviewer  
**File:** `frontend/src/styles.css:5663–5672`

`font-size: 10px; padding: 2px 8px` is a fingertip-precision target.

---

### ⚠️ M7: `get_deviation_score` Subqueries Uncapped

**Found by:** Database Reviewer  
**File:** `backend/src/repositories.rs`

Faculty overload and underutilized batch queries have no `LIMIT`. A school with 200 overloaded faculty returns 200 rows.

---

## 4. Low-Priority / Technical Debt

| Issue | Found By | File |
|-------|----------|------|
| Code duplication between 4 timetable panels (modal shell, filters, week-start math) | Architecture | `components/timetable/*.tsx` |
| `FacultyApp.tsx` is 840 lines, doing too much | Architecture | `components/faculty/FacultyApp.tsx` |
| Alert polling duplicated in `App.tsx` and `FacultyApp.tsx` | Architecture | `App.tsx`, `FacultyApp.tsx` |
| `App.tsx` approaching 2,558 lines | Architecture | `App.tsx` |
| Magic numbers (`24`, `80`, `20`, `50`, `60000`) scattered across codebase | Architecture | Multiple |
| `components.tsx` still 7,563 lines | Architecture | `components.tsx` |
| No offline resilience in FacultyApp | UX | `FacultyApp.tsx` |
| No school name search in Health Dashboard | UX | `HealthDashboardPanel.tsx` |
| Track filter in Compliance is disconnected | UX | `ComplianceAnalyticsPanel.tsx` |
| No side-by-side school comparison for SIP Head | UX | `ComplianceAnalyticsPanel.tsx` |

---

## 5. End-to-End Journey Ratings (Integration & Testing)

| Journey | Status | Blocker |
|---------|--------|---------|
| **A: Principal checks today's schedule** | 🔴 Broken | `day_of_week` mismatch + infinite loop |
| **B: Faculty gets substitution request** | 🔴 Broken | Accept/decline endpoints don't exist |
| **C: SPOC investigates schedule ticket** | ⚠️ Partial | `loadLectureSessions` is hardcoded stub |
| **D: Center Head checks compliance** | 🔴 Broken | API 404 + response shape crash |
| **E: Admin builds weekly timetable** | ⚠️ Partial | No room conflict warning on save |

---

## 6. Additional Integration Findings

### `TimetableHealthStatus.status` Case Mismatch

**Found by:** Integration & Testing Reviewer  
**Files:** `backend/src/repositories.rs`, `frontend/src/components/timetable/HealthDashboardPanel.tsx`

Backend SQL returns lowercase `'green'/'amber'/'red'`. Frontend type is `"Green" | "Amber" | "Red"`. KPI counts use strict equality (`=== "Green"`) and will **all read 0**.

**Fix:** Normalize case on one side (recommended: backend returns Title Case).

### `loadFacultyWeeklySlots` Uses Today's Date, Not Monday

**Found by:** Integration & Testing Reviewer  
**File:** `frontend/src/App.tsx:901–910`

Queries weekly slots with today's date instead of the Monday of the week. Since weekly slots are stored by Monday date, this always returns empty.

**Fix:** Compute Monday date before querying.

### `Ticket` Type Missing Linked Fields

**Found by:** Integration & Testing Reviewer  
**File:** `frontend/src/types.ts:216–236`

Backend adds `linked_grade_level` and `linked_subject` to `Ticket` struct, but frontend `Ticket` type omits them.

**Fix:** Add fields to `types.ts`.

### `WeeklyTimetableSlot` Missing `is_substitution`

**Found by:** Integration & Testing Reviewer  
**File:** `frontend/src/components/timetable/DayAtAGlancePanel.tsx:196–198`

Component relies on `(s as unknown as Record<string,boolean>).is_substitution` — always false/undefined. Red substitution dot never appears.

**Fix:** Add `is_substitution` to `WeeklyTimetableSlot` type or derive from `actual_faculty_user_id`.

---

## 7. What Works Well (Preserve)

| Area | Finding | Council Member |
|------|---------|---------------|
| **Alert engine separation** | `alerts.rs` is properly isolated from HTTP concerns | Architecture |
| **Error handling patterns** | Consistent with codebase (`Result<T,String>` → `AppError`) | Architecture |
| **Frontend XSS safety** | React auto-escapes all rendered data | Security |
| **Browser notification stub** | Low attack surface, no service worker | Security |
| **Parametrized queries** | No SQL injection in new repository functions | Security |
| **Faculty mobile UX** | Swipeable days, next-class banner, color-coded grades | UX |
| **CSV export** | One-click download works | UX |
| **Data-preserving migration** | `column_exists` guards + safe defaults | Database |
| **WAL mode** | Readers don't block writers | Database |
| **MAX_ROWS caps** | Most list queries respect 1,000-row limit | Database |

---

## 6. Council Consensus: Required Actions Before Ship

### Phase 1: Critical Fixes (Do Not Deploy Without)
1. **Rename ticket-linking migration to 43** (`db.rs`)
2. **Align ALL API URLs** between `api.ts` and `routes/mod.rs`
3. **Align response types** between `models.rs` and `types.ts`
4. **Fix `CrossSchoolFacultyPanel` data flow** — stop overwriting slots with `[]`
5. **Scope alert queries** — pass `scope_school_ids` into each check function
6. **Add alert caching** — TTL cache or materialized snapshot
7. **Wrap all `loadXxx` callbacks in `useCallback`** — stop infinite re-render loops
8. **Fix `day_of_week` convention** — Monday=0 across all new components
9. **Add missing `accept_substitution` / `decline_substitution` backend routes**

### Phase 2: High-Priority Fixes (Before User Testing)
10. Fix alert scope leak (`unwrap_or(true)` → `unwrap_or(false)`)
11. Add missing composite indexes
12. Add rate limiting on alert endpoints
13. Refresh `linked_*` columns in `update_ticket`
14. Replace 36-column ordinal SELECT with safe hydration
15. De-duplicate faculty notifications
16. Add accessible text badges to color-coded indicators
17. Fix `TimetableHealthStatus.status` case mismatch (Green vs green)
18. Fix `loadFacultyWeeklySlots` to use Monday date
19. Wire up `loadLectureSessions` — replace stub with real API call
20. Add `linked_grade_level` / `linked_subject` to frontend `Ticket` type

### Phase 3: Polish (Before Production)
21. Scope `check_unfilled_periods` to current week
22. Add 11 AM gate to `check_attendance_not_marked`
23. Fix `check_double_bookings` to query weekly slots
24. Add Hindi/Urdu schedule keywords
25. Add `CHECK` constraint on `session_type`
26. Enlarge Raise Ticket button on mobile
27. Cap `get_deviation_score` subqueries
28. Add `is_substitution` to `WeeklyTimetableSlot` type

---

*Review conducted by 5 specialized council agents:*
1. **Architecture & Code Quality** — Rust/React patterns, maintainability, API contracts
2. **Security & Stability** — Auth scope, SQL injection, DoS risks, data integrity
3. **UX & Usability** — Stakeholder needs, mobile UX, accessibility, navigation
4. **Database & Performance** — Query efficiency, index strategy, SQLite scalability
5. **Integration & Testing** — End-to-end flows, API alignment, state management, edge cases
